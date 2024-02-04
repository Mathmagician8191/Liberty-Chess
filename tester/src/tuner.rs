use array2d::Array2D;
use liberty_chess::{Board, Piece};
use oxidation::parameters::{Parameters, DEFAULT_PARAMETERS};
use oxidation::{evaluate_raw, get_promotion_values};
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::fs::read_to_string;
use std::time::Instant;
use tester::POSITIONS;

const TUNE_K: bool = true;

type GameData = Vec<(Array2D<Piece>, Vec<Piece>, bool, u8, u32, f64)>;

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
  let (loss, positions) = data
    .par_iter()
    .map(
      |(pieces, promotions, to_move, halfmoves, count, game_score)| {
        let mut score = evaluate_raw(
          pieces,
          get_promotion_values(promotions, parameters),
          *to_move,
          *halfmoves,
          parameters,
        );
        if !to_move {
          score = -score;
        }
        let fcount = f64::from(*count);
        let sigmoid = 1.0 / (1.0 + (-k * f64::from(score) / 100.0).exp());
        let loss = fcount * (game_score - sigmoid).powi(2);
        (loss, *count)
      },
    )
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
  println!("{parameters:?}");
  let mut data = Vec::new();
  for (file, _, _, k) in POSITIONS {
    println!("Position {file}");
    let mut processed_data = Vec::new();
    for folder in [
      "verif search take 1",
      "verif search take 2",
      "verif search take 3",
      "cache check",
    ] {
      let fens =
        read_to_string(format!("datagen/{folder}/{file}.txt")).expect("Unable to read file");
      processed_data.extend(fens.split('\n').map(|line| {
        let mut line = line.split(';');
        let board = Board::new(line.next().expect("missing FEN")).expect("Invalid position");
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
        (
          board.board().clone(),
          board.promotion_options().clone(),
          board.to_move(),
          board.halfmoves(),
          games,
          f64::from(score) / f64::from(games) / 2.0,
        )
      }))
    }
    println!("Loaded {} positions", processed_data.len());
    // Calculate K
    if TUNE_K {
      let mut best_k = *k;
      let (loss, positions) = calculate_loss(best_k, &processed_data, &parameters);
      let mut best_loss = loss / f64::from(positions);
      let mut delta = 0.01;
      println!("k {k} loss {best_loss:.6}");
      while delta > 0.0001 {
        let mut changed = false;
        let k = best_k + delta;
        let (loss, positions) = calculate_loss(k, &processed_data, &parameters);
        let average_loss = loss / f64::from(positions);
        if average_loss < best_loss {
          println!("Position {file} k {k} loss {average_loss:.6}");
          best_loss = average_loss;
          best_k = k;
          changed = true;
        } else {
          let k = best_k - delta;
          let (loss, positions) = calculate_loss(k, &processed_data, &parameters);
          let average_loss = loss / f64::from(positions);
          if average_loss < best_loss {
            println!("Position {file} k {k} loss {average_loss:.6}");
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
    } else {
      data.push((*k, processed_data));
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
          } else {
            break;
          }
        }
      }
    }
    println!("Iteration took {}s", start.elapsed().as_secs());
    println!("{parameters:?}");
  }
}
