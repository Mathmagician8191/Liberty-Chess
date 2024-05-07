use liberty_chess::{Board, Piece};
use oxidation::evaluate::{eval_features, extract_features, gradient, Features};
use oxidation::get_promotion_values;
use oxidation::parameters::{Parameters, DEFAULT_PARAMETERS};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::fs::{read_dir, read_to_string};
use std::time::Instant;
use tester::POSITIONS;

const ITERATION_COUNT: i32 = 200;
const PRINT_FREQUENCY: i32 = 25;
const LR: f64 = 20000.0;
const MOMENTUM_FACTOR: f64 = 0.8;

type GameData = Vec<(Features, Vec<Piece>, bool, u32, f64)>;

fn calculate_gradients_batch(
  data: &Vec<(f64, GameData)>,
  parameters: &Parameters<f64>,
) -> (f64, Parameters<f64>) {
  let mut loss_total = 0.0;
  let mut gradient_total = Parameters::default();
  let mut features_total = Parameters::default();
  let mut position_total = 0;
  for (k, data) in data {
    let (loss, gradient, feature_counts, positions) = calculate_gradients(*k, data, parameters);
    loss_total += loss;
    gradient_total += gradient;
    features_total += feature_counts;
    position_total += positions;
  }
  (
    loss_total / f64::from(position_total),
    (gradient_total / features_total).sanitize(),
  )
}

fn calculate_gradients(
  k: f64,
  data: &GameData,
  parameters: &Parameters<f64>,
) -> (f64, Parameters<f64>, Parameters<f64>, u32) {
  data
    .par_iter()
    .map(|(features, promotions, to_move, count, game_score)| {
      let promotion_values = get_promotion_values::<f64>(promotions, parameters);
      let mut score = eval_features(features, *to_move, promotion_values, parameters);
      if !to_move {
        score = -score;
      }
      let fcount = f64::from(*count);
      let exp_score = (-k * score / 100.0).exp();
      let sigmoid = 1.0 / (1.0 + exp_score);
      let loss = fcount * (game_score - sigmoid).powi(2);
      // calculate derivative of loss wrt eval
      let dsigmoid = -k * 0.01 * exp_score / (1.0 + exp_score).powi(2);
      let loss_gradient = -2.0 * fcount * (game_score - sigmoid) * dsigmoid;
      let raw_gradient = gradient(features.clone(), promotion_values, parameters);
      let gradient = raw_gradient * loss_gradient;
      (loss, gradient, raw_gradient.abs(), *count)
    })
    .reduce(
      || (0.0, Parameters::default(), Parameters::default(), 0),
      |(loss_acc, gradient_acc, features_acc, count_acc), (loss, gradient, features, count)| {
        (
          loss_acc + loss,
          gradient_acc + gradient,
          features_acc + features,
          count_acc + count,
        )
      },
    )
}

fn calculate_loss(k: f64, data: &GameData, parameters: &Parameters<i32>) -> (f64, u32) {
  data
    .par_iter()
    .map(|(features, promotions, to_move, count, game_score)| {
      let mut score = eval_features(
        features,
        *to_move,
        get_promotion_values(promotions, parameters),
        parameters,
      );
      if !to_move {
        score = -score;
      }
      let fcount = f64::from(*count);
      let sigmoid = 1.0 / (1.0 + (-k * f64::from(score) / 100.0).exp());
      let loss = fcount * (game_score - sigmoid).powi(2);
      (loss, *count)
    })
    .reduce(
      || (0.0, 0),
      |(loss_acc, count_acc), (loss, count)| (loss_acc + loss, count_acc + count),
    )
}

fn setup_threadpool() {
  // stop the stack overflows
  ThreadPoolBuilder::new()
    .stack_size(1 << 23)
    .build_global()
    .expect("Failed to set stack size");
}

fn process_position(position: &&str, data: &mut Vec<(f64, GameData)>, total_positions: &mut usize) {
  println!("Position {position}");
  let mut processed_data = Vec::new();
  let folders = read_dir("datagen/Old").expect("Folder does not exist");
  for folder in folders {
    let mut folder = folder.expect("Folder does not exist").path();
    folder.push(format!("{position}.txt"));
    let fens = read_to_string(folder).expect("Unable to read file");
    let lines: Vec<&str> = fens.split('\n').collect();
    processed_data.extend(
      lines
        .par_iter()
        .flat_map(|line| {
          let mut line = line.split(';');
          let board = Board::new(line.next().expect("missing FEN")).expect("Invalid position");
          if board.halfmoves() >= 30 || board.in_check() {
            return None;
          }
          let games: u32 = line
            .next()
            .expect("Missing games")
            .parse()
            .expect("Invalid game count");
          let score: u32 = line
            .next()
            .expect("Missing score")
            .parse()
            .expect("Invalid score");
          let features = extract_features(board.board());
          Some((
            features,
            board.promotion_options().clone(),
            board.to_move(),
            games,
            f64::from(score) / f64::from(games) / 2.0,
          ))
        })
        .collect::<GameData>(),
    );
  }
  let position_count = processed_data.len();
  println!("Loaded {position_count} positions");
  *total_positions += position_count;
  // Calculate K
  let mut best_k = 0.0;
  let (loss, positions) = calculate_loss(best_k, &processed_data, &DEFAULT_PARAMETERS);
  let mut best_loss = loss / f64::from(positions);
  let mut delta = 0.09;
  println!("k {best_k} loss {best_loss:.6}");
  while delta > 0.0001 {
    let mut changed = false;
    let k = best_k + delta;
    let (loss, positions) = calculate_loss(k, &processed_data, &DEFAULT_PARAMETERS);
    let average_loss = loss / f64::from(positions);
    if average_loss < best_loss {
      println!("Position {position} k {k} loss {average_loss:.6}");
      best_loss = average_loss;
      best_k = k;
      changed = true;
    } else {
      let k = best_k - delta;
      let (loss, positions) = calculate_loss(k, &processed_data, &DEFAULT_PARAMETERS);
      let average_loss = loss / f64::from(positions);
      if average_loss < best_loss {
        println!("Position {position} k {k} loss {average_loss:.6}");
        best_loss = average_loss;
        best_k = k;
        changed = true;
      }
    }
    if !changed {
      delta /= 3.0;
    }
  }
  println!("Final k for {position}: {best_k}");
  data.push((best_k, processed_data));
}

fn main() {
  setup_threadpool();
  let mut parameters = Parameters {
    pieces: DEFAULT_PARAMETERS
      .pieces
      .map(|(x, y)| (f64::from(x), f64::from(y))),
    pawn_scale_factor: f64::from(DEFAULT_PARAMETERS.pawn_scale_factor),
    pawn_scaling_bonus: f64::from(DEFAULT_PARAMETERS.pawn_scaling_bonus),
    ..Default::default()
  };
  let mut data = Vec::new();
  let mut total_positions = 0;
  let mut start = Instant::now();
  for (position, _, _) in POSITIONS {
    process_position(position, &mut data, &mut total_positions);
  }
  println!("{total_positions} positions in dataset");
  let mut best_loss = f64::INFINITY;
  println!("Data loading took {}s", start.elapsed().as_secs());
  start = Instant::now();
  // Tune parameters using Nesterov momentum
  let mut momentum = Parameters::default();
  for i in 0..=ITERATION_COUNT {
    parameters += momentum;
    parameters.enforce_invariants();
    let (loss, mut gradient) = calculate_gradients_batch(&data, &parameters);
    if loss < best_loss {
      best_loss = loss;
      println!("Iteration {i}/{ITERATION_COUNT} Loss record {loss:.7}");
    } else {
      println!("Iteration {i}/{ITERATION_COUNT} Loss {loss:.7} (Best: {best_loss:.7})");
    }
    gradient = gradient * LR;
    momentum = (gradient + momentum) * MOMENTUM_FACTOR;
    parameters += gradient;
    parameters.enforce_invariants();
    if i % PRINT_FREQUENCY == 0 {
      println!("{parameters:?}");
      println!(
        "{PRINT_FREQUENCY} Iterations took {}s",
        start.elapsed().as_secs()
      );
      start = Instant::now();
    }
  }
  println!("{parameters:?}");
  println!("{}", parameters.to_string());
}
