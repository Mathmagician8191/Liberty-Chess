#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A chess engine for Liberty Chess

use liberty_chess::moves::Move;
use liberty_chess::{Board, Gamestate};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::max;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use ulci::client::Message;
use ulci::{OptionValue, Score};

/// Internal naming thing - do not use
pub const QDEPTH_NAME: &str = "QDepth";

const PIECE_VALUES: [f64; 18] = [
  100.0,  // pawn
  320.0,  // knight
  330.0,  // bishop
  500.0,  // rook
  975.0,  // queen
  0.0,    // king
  880.0,  // archbishop
  975.0,  // chancellor
  200.0,  // camel
  300.0,  // zebra
  300.0,  // mann
  450.0,  // nightrider
  700.0,  // champion
  700.0,  // centaur
  1350.0, // amazon
  800.0,  // elephant
  25.0,   // obstacle
  75.0,   // wall
];

/// Configuration for the search
pub struct SearchConfig<'a> {
  qdepth: &'a mut u8,
  start: Instant,
  max_depth: u8,
  max_time: u128,
  max_nodes: usize,
  rx: &'a Receiver<Message>,
  stopped: bool,
  can_stop: bool,
  nodes: usize,
  debug: &'a mut bool,
}

impl<'a> SearchConfig<'a> {
  /// Initialise the search config
  pub fn new(
    qdepth: &'a mut u8,
    max_depth: u8,
    max_time: u128,
    max_nodes: usize,
    rx: &'a Receiver<Message>,
    debug: &'a mut bool,
  ) -> Self {
    Self {
      qdepth,
      start: Instant::now(),
      max_depth,
      max_time,
      max_nodes,
      rx,
      stopped: false,
      can_stop: false,
      nodes: 0,
      debug,
    }
  }

  fn search_is_over(&mut self) -> bool {
    if !self.can_stop {
      return false;
    }
    if self.stopped
      || self.nodes >= self.max_nodes
      || self.start.elapsed().as_millis() >= self.max_time
    {
      self.stopped = true;
      return true;
    }
    if self.nodes % 128 == 0 {
      while let Ok(message) = self.rx.try_recv() {
        match message {
          Message::SetDebug(new_debug) => *self.debug = new_debug,
          Message::UpdatePosition(_) => {
            if *self.debug {
              println!("info string servererror search in progress");
            }
          }
          Message::Go(_) => {
            if *self.debug {
              println!("info string servererror already searching");
            }
          }
          Message::Stop => {
            self.stopped = true;
            return true;
          }
          Message::UpdateOption(name, value) => {
            if name == QDEPTH_NAME {
              match value {
                OptionValue::UpdateInt(value) => *self.qdepth = value as u8,
                OptionValue::SendTrigger
                | OptionValue::UpdateBool(_)
                | OptionValue::UpdateRange(_)
                | OptionValue::UpdateString(_) => {
                  if *self.debug {
                    // TODO: make it use output correctly
                    println!("info string servererror incorrect option type");
                  }
                }
              }
            }
          }
        }
      }
    }
    false
  }
}

/// Returns a random legal move from the provided position, if one exists
#[must_use]
pub fn random_move(board: &Board) -> Option<Move> {
  let moves = board.generate_legal();
  moves.choose(&mut thread_rng())?.last_move
}

/// Returns an evaluation of the provided position
#[must_use]
fn evaluate(board: &Board) -> Score {
  match board.state() {
    Gamestate::InProgress => {
      let mut score = 0.0;
      for piece in board.board().elements_row_major_iter() {
        if *piece != 0 {
          let multiplier = if *piece > 0 { 1.0 } else { -1.0 };
          let value = PIECE_VALUES[piece.unsigned_abs() as usize - 1];
          score += value * multiplier;
        }
      }
      if !board.to_move() {
        score *= -1.0;
      }
      Score::Centipawn(score)
    }
    Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
      Score::Centipawn(0.0)
    }
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(0),
  }
}

fn quiescence(
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score) {
  let score = evaluate(board);
  settings.nodes += 1;
  if score >= beta {
    return (Vec::new(), beta);
  }
  let mut best_pv = Vec::new();
  if alpha < score {
    alpha = score;
  }
  if settings.search_is_over() {
    return (best_pv, alpha);
  }
  if (depth != 0) && (board.state() == Gamestate::InProgress) {
    for position in board.generate_legal_quiescence() {
      let (mut pv, mut score) = quiescence(settings, &position, depth - 1, !beta, !alpha);
      score = -score;
      if score >= beta {
        return (Vec::new(), beta);
      }
      if score > alpha {
        alpha = score;
        if let Some(bestmove) = position.last_move {
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
      }
      if settings.search_is_over() {
        return (best_pv, alpha);
      }
    }
  }
  (best_pv, alpha)
}

fn alpha_beta(
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score) {
  if depth == 0 || board.state() != Gamestate::InProgress {
    quiescence(settings, board, *settings.qdepth, alpha, beta)
  } else {
    let mut best_pv = Vec::new();
    let (mut moves, mut other) = board.generate_legal_buckets();
    moves.append(&mut other);
    for position in moves {
      let (mut pv, mut score) = alpha_beta(settings, &position, depth - 1, !beta, !alpha);
      score = -score;
      if score >= beta {
        return (Vec::new(), beta);
      }
      if score > alpha {
        alpha = score;
        if let Some(bestmove) = position.last_move {
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
      }
      if settings.search_is_over() {
        return (best_pv, alpha);
      }
    }
    (best_pv, alpha)
  }
}

fn alpha_beta_root(
  settings: &mut SearchConfig,
  board: &Board,
  moves: &Vec<Move>,
  depth: u8,
) -> (Vec<Move>, Score) {
  let mut alpha = Score::Loss(0);
  let mut best_pv = Vec::new();
  for candidate in moves {
    if let Some(position) = board.move_if_legal(*candidate) {
      let (mut pv, mut score) = alpha_beta(settings, &position, depth - 1, Score::Loss(0), !alpha);
      if settings.search_is_over() {
        return (best_pv, alpha);
      }
      score = -score;
      if score > alpha {
        alpha = score;
        if let Some(bestmove) = position.last_move {
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
          settings.can_stop = true;
        }
      }
    }
  }
  (best_pv, alpha)
}

/// Search the specified position and moves to the specified depth
pub fn search(mut settings: SearchConfig, position: &Board, mut moves: Vec<Move>) -> Vec<Move> {
  let mut best_pv = Vec::new();
  let mut depth = 0;
  let mut current_score = Score::Loss(0);
  while depth < settings.max_depth {
    depth += 1;
    let (pv, score) = alpha_beta_root(&mut settings, position, &moves, depth);
    let time = settings.start.elapsed().as_millis();
    let nps = (1000 * settings.nodes) / max(time as usize, 1);
    if !pv.is_empty() {
      best_pv = pv;
      current_score = score;
    }
    // TODO: make it use output correctly
    println!(
      "info depth {depth} score {} time {time} nodes {} nps {nps} pv {}",
      current_score.to_string(),
      settings.nodes,
      best_pv
        .iter()
        .map(Move::to_string)
        .collect::<Vec<String>>()
        .join(" ")
    );
    if settings.search_is_over() {
      break;
    }
    let bestmove = best_pv[0];
    let mut sorted_moves = vec![bestmove];
    sorted_moves.append(
      &mut moves
        .into_iter()
        .filter(|m| *m != bestmove)
        .collect::<Vec<Move>>(),
    );
    moves = sorted_moves;
  }
  best_pv
}
