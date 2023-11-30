use liberty_chess::threading::CompressedBoard;
use liberty_chess::Board;
use oxidation::parameters::{
  ENDGAME_EDGE_AVOIDANCE, ENDGAME_PIECE_VALUES, MIDDLEGAME_EDGE_AVOIDANCE, MIDDLEGAME_PIECE_VALUES,
};
use oxidation::{quiescence, SearchConfig, State, QDEPTH};
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::fs::read_to_string;
use std::sync::mpsc::channel;
use std::time::Instant;
use tester::POSITIONS;
use ulci::{Score, SearchTime};

const TUNE_K: bool = true;
const MAX_ITERATIONS: usize = 10;

type GameData = Vec<(CompressedBoard, u32, f64)>;

fn calculate_loss_batch(
  data: &Vec<(f64, GameData)>,
  mg_piece_values: &[i32; 18],
  mg_edge_avoidance: &[i32; 18],
  eg_piece_values: &[i32; 18],
  eg_edge_avoidance: &[i32; 18],
) -> f64 {
  let mut loss_total = 0.0;
  let mut position_total = 0;
  for (k, data) in data {
    let (loss, positions) = calculate_loss(
      *k,
      data,
      mg_piece_values,
      mg_edge_avoidance,
      eg_piece_values,
      eg_edge_avoidance,
    );
    loss_total += loss;
    position_total += positions;
  }
  loss_total / f64::from(position_total)
}

fn calculate_loss(
  k: f64,
  data: &GameData,
  mg_piece_values: &[i32; 18],
  mg_edge_avoidance: &[i32; 18],
  eg_piece_values: &[i32; 18],
  eg_edge_avoidance: &[i32; 18],
) -> (f64, u32) {
  let state = State::new(0);
  let (loss, positions) = data
    .par_iter()
    .map(|(position, count, game_score)| {
      let mut qdepth = QDEPTH;
      let mut debug = false;
      let (_tx, rx) = channel();
      let mut settings = SearchConfig::new_time(&mut qdepth, SearchTime::Infinite, &rx, &mut debug);
      let position = position.clone().load_from_thread();
      let (_pv, mut score) = quiescence(
        &state,
        &mut settings,
        &position,
        3,
        Score::Loss(0),
        Score::Win(0),
        mg_piece_values,
        mg_edge_avoidance,
        eg_piece_values,
        eg_edge_avoidance,
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
  let mut mg_piece_values = MIDDLEGAME_PIECE_VALUES;
  let mut mg_edge_avoidance = MIDDLEGAME_EDGE_AVOIDANCE;
  let mut eg_piece_values = ENDGAME_PIECE_VALUES;
  let mut eg_edge_avoidance = ENDGAME_EDGE_AVOIDANCE;
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
      let (loss, positions) = calculate_loss(
        best_k,
        &processed_data,
        &mg_piece_values,
        &mg_edge_avoidance,
        &eg_piece_values,
        &eg_edge_avoidance,
      );
      let mut best_loss = loss / f64::from(positions);
      let mut delta = 0.01;
      println!("Position {file} k {k} loss {best_loss:.4}");
      while delta > 0.0001 {
        let mut changed = false;
        let k = best_k + delta;
        let (loss, positions) = calculate_loss(
          k,
          &processed_data,
          &mg_piece_values,
          &mg_edge_avoidance,
          &eg_piece_values,
          &eg_edge_avoidance,
        );
        let average_loss = loss / f64::from(positions);
        println!("Position {file} k {k} loss {average_loss:.4}");
        if average_loss < best_loss {
          best_loss = average_loss;
          best_k = k;
          changed = true;
        } else {
          let k = best_k - delta;
          let (loss, positions) = calculate_loss(
            k,
            &processed_data,
            &mg_piece_values,
            &mg_edge_avoidance,
            &eg_piece_values,
            &eg_edge_avoidance,
          );
          let average_loss = loss / f64::from(positions);
          println!("Position {file} k {k} loss {average_loss:.4}");
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
  let mut best_loss = calculate_loss_batch(
    &data,
    &mg_piece_values,
    &mg_edge_avoidance,
    &eg_piece_values,
    &eg_edge_avoidance,
  );
  println!("{}ms to calculate loss", start.elapsed().as_millis());
  // Tune parameters
  let mut parameters: Vec<usize> = (0..72).collect();
  let mut iteration_count = 0;
  let mut changed = true;
  while changed {
    let start = Instant::now();
    iteration_count += 1;
    println!("Starting iteration {iteration_count}, Loss {best_loss:.5}");
    parameters.shuffle(&mut thread_rng());
    changed = false;
    for parameter in &parameters {
      let parameter = *parameter;
      // try increasing the parameter
      let mut updated = false;
      for _ in 0..MAX_ITERATIONS {
        let mut new_piece_mg = mg_piece_values;
        let mut new_edge_mg = mg_edge_avoidance;
        let mut new_piece_eg = eg_piece_values;
        let mut new_edge_eg = eg_edge_avoidance;
        let index = parameter % 18;
        if parameter >= 54 {
          new_piece_mg[index] += 1;
        } else if parameter >= 36 {
          new_edge_mg[index] += 1;
        } else if parameter >= 18 {
          new_piece_eg[index] += 1;
        } else {
          new_edge_eg[index] += 1;
        }
        let new_loss = calculate_loss_batch(
          &data,
          &new_piece_mg,
          &new_edge_mg,
          &new_piece_eg,
          &new_edge_eg,
        );
        if new_loss < best_loss {
          best_loss = new_loss;
          mg_piece_values = new_piece_mg;
          mg_edge_avoidance = new_edge_mg;
          eg_piece_values = new_piece_eg;
          eg_edge_avoidance = new_edge_eg;
          changed = true;
          updated = true;
        } else {
          break;
        }
      }
      if !updated {
        for _ in 0..MAX_ITERATIONS {
          let mut new_piece_mg = mg_piece_values;
          let mut new_edge_mg = mg_edge_avoidance;
          let mut new_piece_eg = eg_piece_values;
          let mut new_edge_eg = eg_edge_avoidance;
          let index = parameter % 18;
          if parameter >= 54 {
            new_piece_mg[index] -= 1;
          } else if parameter >= 36 {
            new_edge_mg[index] -= 1;
          } else if parameter >= 18 {
            new_piece_eg[index] -= 1;
          } else {
            new_edge_eg[index] -= 1;
          }
          let new_loss = calculate_loss_batch(
            &data,
            &new_piece_mg,
            &new_edge_mg,
            &new_piece_eg,
            &new_edge_eg,
          );
          if new_loss < best_loss {
            best_loss = new_loss;
            mg_piece_values = new_piece_mg;
            mg_edge_avoidance = new_edge_mg;
            eg_piece_values = new_piece_eg;
            eg_edge_avoidance = new_edge_eg;
            changed = true;
          } else {
            break;
          }
        }
      }
    }
    println!("Iteration took {}s", start.elapsed().as_secs());
    println!("{mg_piece_values:?}");
    println!("{mg_edge_avoidance:?}");
    println!("{eg_piece_values:?}");
    println!("{eg_edge_avoidance:?}");
  }
}
