#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A chess engine for Liberty Chess

use liberty_chess::moves::Move;
use liberty_chess::{Board, Gamestate};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::max;
use std::time::Instant;
use ulci::Score;

/// Returns a random legal move from the provided position, if one exists
#[must_use]
pub fn random_move(board: &Board) -> Option<Move> {
  let moves = board.generate_legal();
  moves.choose(&mut thread_rng())?.last_move
}

/// Returns an evaluation of the provided position
#[must_use]
pub const fn evaluate(board: &Board) -> Score {
  match board.state() {
    Gamestate::InProgress => {
      let (white_pieces, black_pieces) = board.pieces();
      let (to_move_pieces, other_pieces) = if board.to_move() {
        (white_pieces as isize, black_pieces as isize)
      } else {
        (black_pieces as isize, white_pieces as isize)
      };
      let score = to_move_pieces - other_pieces;
      Score::Centipawn((score * 100) as f64)
    }
    Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
      Score::Centipawn(0.0)
    }
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(0),
  }
}

fn alpha_beta(
  board: &Board,
  depth: u16,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score, usize) {
  if (depth == 0) || (board.state() != Gamestate::InProgress) {
    let mut score = evaluate(board);
    if score >= beta {
      score = beta;
    }
    if alpha < score {
      alpha = score;
    }
    (Vec::new(), alpha, 1)
  } else {
    let mut best_pv = Vec::new();
    let mut nodes = 0;
    for position in board.generate_legal() {
      let (mut pv, mut score, child_nodes) = alpha_beta(&position, depth - 1, !beta, !alpha);
      nodes += child_nodes;
      score = -score;
      if score >= beta {
        return (Vec::new(), beta, nodes);
      }
      if score > alpha {
        alpha = score;
        if let Some(bestmove) = position.last_move {
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
      }
    }
    (best_pv, alpha, nodes)
  }
}

fn alpha_beta_root(board: &Board, moves: Vec<Move>, depth: u16) -> (Vec<Move>, Score, usize) {
  let mut alpha = Score::Loss(0);
  let mut best_pv = Vec::new();
  let mut nodes = 0;
  for candidate in moves {
    if let Some(position) = board.move_if_legal(candidate) {
      let (mut pv, mut score, child_nodes) =
        alpha_beta(&position, depth - 1, Score::Loss(0), !alpha);
      nodes += child_nodes;
      score = -score;
      if score > alpha {
        alpha = score;
        if let Some(bestmove) = position.last_move {
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
      }
    }
  }
  (best_pv, alpha, nodes)
}

/// Search the specified position and moves to the specified depth
pub fn search(
  start: &Instant,
  position: &Board,
  depth: u16,
  moves: &[Move],
  nodes: &mut usize,
) -> Vec<Move> {
  let (pv, score, new_nodes) = alpha_beta_root(position, moves.to_vec(), depth);
  *nodes += new_nodes;
  let time = start.elapsed().as_millis();
  let nps = (1000 * *nodes) / max(time as usize, 1);
  println!(
    "info depth {depth} score {} time {time} nodes {nodes} nps {nps} pv {}",
    score.to_string(),
    pv.iter()
      .map(Move::to_string)
      .collect::<Vec<String>>()
      .join(" ")
  );
  pv
}
