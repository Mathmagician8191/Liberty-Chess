use liberty_chess::positions::get_startpos;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::Board;
use oxidation::parameters::{Parameters, DEFAULT_PARAMETERS};
use oxidation::{quiescence, SearchConfig, State, QDEPTH};
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::fs::read_to_string;
use std::sync::mpsc::channel;
use std::time::Instant;
use tester::POSITIONS;
use ulci::{Score, SearchTime};

const TUNE_K: bool = true;

type GameData = Vec<(CompressedBoard, u32, f64)>;

fn calculate_loss_batch(data: &Vec<(f64, GameData)>, parameters: &Parameters) -> f64 {
  let mut loss_total = 0.0;
  let mut position_total = 0;
  for (k, data) in data {
    let (loss, positions) = calculate_loss(*k, data, parameters);
    loss_total += loss;
    position_total += positions;
  }
  loss_total / f64::from(position_total)
}

fn calculate_loss(k: f64, data: &GameData, parameters: &Parameters) -> (f64, u32) {
  let state = State::new(0, &get_startpos());
  let (loss, positions) = data
    .par_iter()
    .map(|(position, count, game_score)| {
      let mut qdepth = QDEPTH;
      let mut debug = false;
      let (_tx, rx) = channel();
      let position = position.clone().load_from_thread();
      let mut settings = SearchConfig::new_time(
        &position,
        &mut qdepth,
        SearchTime::Infinite,
        &rx,
        &mut debug,
      );
      let (_pv, mut score) = quiescence(
        &state,
        &mut settings,
        &position,
        3,
        Score::Loss(0),
        Score::Win(0),
        parameters,
      );
      if !position.to_move() {
        score = -score;
      }
      if let Score::Centipawn(score) = score {
        let fcount = f64::from(*count);
        let sigmoid = 1.0 / (1.0 + (-k * f64::from(score) / 100.0).exp());
        let loss = fcount * (game_score - sigmoid).powi(2);
        (loss, *count)
      } else {
        (0.0, 0)
      }
    })
    .fold(
      || (0.0, 0),
      |(loss_acc, count_acc), (loss, count)| (loss_acc + loss, count_acc + count),
    )
    .reduce(
      || (0.0, 0),
      |(loss_acc, count_acc), (loss, count)| (loss_acc + loss, count_acc + count),
    );
  (loss, positions)
}

fn main() {
  let mut parameters = DEFAULT_PARAMETERS;
  let mut data = Vec::new();
  for (file, _, _, k) in POSITIONS {
    let fens = read_to_string(format!("datagen/{file}.txt")).expect("Unable to read file");
    let processed_data: GameData = fens
      .split('\n')
      .map(|line| {
        let mut line = line.split(';');
        let board = Board::new(line.next().expect("missing FEN"))
          .expect("Invalid position")
          .send_to_thread();
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
        (board, games, f64::from(score) / f64::from(games) / 2.0)
      })
      .collect();
    // Calculate K
    if TUNE_K {
      let mut best_k = *k;
      let (loss, positions) = calculate_loss(best_k, &processed_data, &parameters);
      let mut best_loss = loss / f64::from(positions);
      let mut delta = 0.01;
      println!("Position {file} k {k} loss {best_loss:.6}");
      while delta > 0.0001 {
        let mut changed = false;
        let k = best_k + delta;
        let (loss, positions) = calculate_loss(k, &processed_data, &parameters);
        let average_loss = loss / f64::from(positions);
        println!("Position {file} k {k} loss {average_loss:.6}");
        if average_loss < best_loss {
          best_loss = average_loss;
          best_k = k;
          changed = true;
        } else {
          let k = best_k - delta;
          let (loss, positions) = calculate_loss(k, &processed_data, &parameters);
          let average_loss = loss / f64::from(positions);
          println!("Position {file} k {k} loss {average_loss:.6}");
          if average_loss < best_loss {
            best_loss = average_loss;
            best_k = k;
            changed = true;
          }
        }
        if !changed {
          delta /= 3.0;
        }
      }
      println!("Final k for {file}: {best_k}");
      data.push((best_k, processed_data));
    }
  }
  let start = Instant::now();
  let mut best_loss = calculate_loss_batch(&data, &parameters);
  println!("{}ms to calculate loss", start.elapsed().as_millis());
  // Tune parameters
  let mut parameter_indices: Vec<usize> = (0..Parameters::COUNT).collect();
  let mut iteration_count = 0;
  let mut changed = true;
  while changed {
    let start = Instant::now();
    iteration_count += 1;
    println!("Starting iteration {iteration_count}, Loss {best_loss:.6}");
    parameter_indices.shuffle(&mut thread_rng());
    changed = false;
    for parameter in &parameter_indices {
      let parameter = *parameter;
      if !Parameters::valid_index(parameter) {
        continue;
      }
      let iteration_count = Parameters::iteration_count(parameter);
      // try increasing the parameter
      let mut updated = false;
      for x in 1..=iteration_count {
        let mut new_parameters = parameters;
        new_parameters.set_parameter(parameter, x);
        let new_loss = calculate_loss_batch(&data, &new_parameters);
        if new_loss < best_loss {
          best_loss = new_loss;
          parameters = new_parameters;
          changed = true;
          updated = true;
        } else {
          break;
        }
      }
      if !updated {
        for x in 1..=iteration_count {
          let mut new_parameters = parameters;
          new_parameters.set_parameter(parameter, -x);
          let new_loss = calculate_loss_batch(&data, &new_parameters);
          if new_loss < best_loss {
            best_loss = new_loss;
            parameters = new_parameters;
            changed = true;
            updated = true;
          } else {
            break;
          }
        }
      }
      if updated {
        println!("Loss {best_loss:.6}");
        println!("{parameters:?}");
      }
    }
    println!("Iteration took {}s", start.elapsed().as_secs());
  }
}
