#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A chess engine for Liberty Chess

use liberty_chess::moves::Move;
use liberty_chess::{Board, Gamestate};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::max;
use std::io::{Stdout, Write};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError::Disconnected;
use std::time::Instant;
use tt::{Entry, ScoreType, TranspositionTable};
use ulci::client::Message;
use ulci::{OptionValue, Score, SearchTime};

/// Interface for efficiently integrating into another application
pub mod glue;

mod tt;

/// The version number of the engine
pub const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");

/// Default Quiescence depth
pub const QDEPTH: u8 = 3;
/// Default Hash size
pub const HASH_SIZE: usize = 64;

/// Internal naming thing - do not use
pub const QDEPTH_NAME: &str = "QDepth";
/// Internal naming thing - do not use
pub const HASH_NAME: &str = "Hash";

const PIECE_VALUES: [i32; 18] = [
  100,  // pawn
  320,  // knight
  330,  // bishop
  500,  // rook
  975,  // queen
  0,    // king
  880,  // archbishop
  975,  // chancellor
  200,  // camel
  300,  // zebra
  300,  // mann
  450,  // nightrider
  700,  // champion
  700,  // centaur
  1350, // amazon
  800,  // elephant
  25,   // obstacle
  75,   // wall
];

/// The state of the engine
pub struct State {
  table: TranspositionTable,
}

impl State {
  /// Initialise a new state, sets up a TT of the provided capacity
  pub fn new(megabytes: usize) -> Self {
    Self {
      table: TranspositionTable::new(megabytes),
    }
  }
}

/// Configuration for the search
pub struct SearchConfig<'a> {
  qdepth: &'a mut u8,
  start: Instant,
  max_depth: u8,
  max_time: u128,
  max_nodes: usize,
  rx: &'a Receiver<Message>,
  stopped: bool,
  nodes: usize,
  debug: &'a mut bool,
  // maximum ply count reached
  seldepth: u16,
}

impl<'a> SearchConfig<'a> {
  /// Initialise the search config
  fn new(
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
      nodes: 0,
      debug,
      seldepth: 0,
    }
  }

  /// Initialise the search config based on the search time
  pub fn new_time(
    qdepth: &'a mut u8,
    time: SearchTime,
    rx: &'a Receiver<Message>,
    debug: &'a mut bool,
  ) -> Self {
    match time {
      SearchTime::Increment(time, inc) => {
        let time = time.saturating_sub(100);
        let time = time.min(time / 20 + inc / 2);
        let time = 1.max(time);
        Self::new(qdepth, u8::MAX, time, usize::MAX, rx, debug)
      }
      SearchTime::Infinite => Self::new(qdepth, u8::MAX, u128::MAX, usize::MAX, rx, debug),
      SearchTime::Other(limits) => {
        Self::new(qdepth, limits.depth, limits.time, limits.nodes, rx, debug)
      }
    }
  }

  fn search_is_over(&mut self) -> bool {
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
      // channel has hung up due to quit or input EOF, stop search
      if matches!(self.rx.try_recv(), Err(Disconnected)) {
        self.stopped = true;
        return true;
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

/// Sort the searchmoves from a position
#[must_use]
pub fn get_move_order(position: &Board, searchmoves: &Vec<Move>) -> Vec<Move> {
  let (captures, other) = position.generate_pseudolegal();
  let (mut captures, mut other): (Vec<(Move, u8, u8)>, Vec<Move>) = if searchmoves.is_empty() {
    (captures, other)
  } else {
    (
      captures
        .into_iter()
        .filter(|(m, _, _)| searchmoves.contains(m))
        .collect(),
      other
        .into_iter()
        .filter(|m| searchmoves.contains(m))
        .collect(),
    )
  };
  captures.shuffle(&mut thread_rng());
  captures.sort_by_key(|(_, piece, capture)| {
    PIECE_VALUES[usize::from(*piece - 1)] - PIECE_VALUES[usize::from(*capture - 1)]
  });
  other.shuffle(&mut thread_rng());
  let mut moves: Vec<Move> = captures.into_iter().map(|(m, _, _)| m).collect();
  moves.append(&mut other);
  moves
}

#[must_use]
fn ply_count(board: &Board) -> u16 {
  board.moves() * 2 + u16::from(!board.to_move())
}

// Returns an evaluation of the provided position
#[must_use]
fn evaluate(board: &Board) -> Score {
  match board.state() {
    Gamestate::InProgress => {
      let mut score = 0;
      for piece in board.board().elements_row_major_iter() {
        if *piece != 0 {
          let multiplier = if *piece > 0 { 1 } else { -1 };
          let value = PIECE_VALUES[piece.unsigned_abs() as usize - 1];
          score += value * multiplier;
        }
      }
      if !board.to_move() {
        score *= -1;
      }
      Score::Centipawn(score)
    }
    Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
      Score::Centipawn(0)
    }
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(board.moves()),
  }
}

fn quiescence(
  state: &State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score) {
  let hash = board.hash();
  if board.state() == Gamestate::InProgress {
    if let Some((pv, score)) = state.table.get(hash, board.moves(), alpha, beta, 0) {
      return (pv, score);
    }
  }
  let score = evaluate(board);
  settings.nodes += 1;
  settings.seldepth = max(settings.seldepth, ply_count(board));
  if score >= beta {
    return (Vec::new(), beta);
  }
  let mut best_pv = Vec::new();
  if alpha < score {
    alpha = score;
  }
  if !settings.search_is_over() && (depth != 0) && (board.state() == Gamestate::InProgress) {
    let mut moves = board.generate_qsearch();
    moves.sort_by_key(|(_, piece, capture)| {
      PIECE_VALUES[usize::from(*piece - 1)] - PIECE_VALUES[usize::from(*capture - 1)]
    });
    for (bestmove, _, _) in moves {
      if let Some(position) = board.test_move_legality(bestmove) {
        let (mut pv, mut score) = quiescence(state, settings, &position, depth - 1, -beta, -alpha);
        score = -score;
        if score >= beta {
          return (Vec::new(), beta);
        }
        if score > alpha {
          alpha = score;
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
        if settings.search_is_over() {
          return (best_pv, alpha);
        }
      }
    }
  }
  (best_pv, alpha)
}

fn alpha_beta(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  mut depth: u8,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score) {
  if board.in_check() {
    depth += 1;
  }
  if board.state() != Gamestate::InProgress {
    quiescence(state, settings, board, *settings.qdepth, alpha, beta)
  } else if depth == 0 {
    let (pv, score) = quiescence(state, settings, board, *settings.qdepth, alpha, beta);
    let (tt_flag, tt_score) = if score >= beta {
      (ScoreType::LowerBound, beta)
    } else if settings.search_is_over() {
      (ScoreType::LowerBound, score)
    } else if score > alpha {
      (ScoreType::Exact, score)
    } else {
      (ScoreType::UpperBound, alpha)
    };
    state.table.store(Entry {
      hash: board.hash(),
      depth: 0,
      movecount: board.moves(),
      scoretype: tt_flag,
      score: tt_score,
      bestmove: pv.get(0).copied(),
    });
    (pv, score)
  } else {
    let hash = board.hash();
    if let Some((pv, score)) = state.table.get(hash, board.moves(), alpha, beta, depth) {
      return (pv, score);
    }
    let mut best_pv = Vec::new();
    let (mut moves, mut other) = board.generate_pseudolegal();
    moves.sort_by_key(|(_, piece, capture)| {
      PIECE_VALUES[usize::from(*piece - 1)] - PIECE_VALUES[usize::from(*capture - 1)]
    });
    let mut moves: Vec<Move> = moves.into_iter().map(|(m, _, _)| m).collect();
    moves.append(&mut other);
    for bestmove in moves {
      if let Some(position) = board.test_move_legality(bestmove) {
        let (mut pv, mut score) = alpha_beta(state, settings, &position, depth - 1, -beta, -alpha);
        score = -score;
        if score >= beta {
          state.table.store(Entry {
            hash,
            depth,
            movecount: board.moves(),
            scoretype: ScoreType::LowerBound,
            score: beta,
            bestmove: None,
          });
          return (Vec::new(), beta);
        }
        if score > alpha {
          alpha = score;
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
        if settings.search_is_over() {
          if !best_pv.is_empty() {
            state.table.store(Entry {
              hash,
              depth,
              movecount: board.moves(),
              scoretype: ScoreType::LowerBound,
              score: alpha,
              bestmove: best_pv.get(0).copied(),
            });
          }
          return (best_pv, alpha);
        }
      }
    }
    let scoretype = if best_pv.is_empty() {
      ScoreType::UpperBound
    } else {
      ScoreType::Exact
    };
    state.table.store(Entry {
      hash,
      depth,
      movecount: board.moves(),
      scoretype,
      score: alpha,
      bestmove: best_pv.get(0).copied(),
    });
    (best_pv, alpha)
  }
}

fn alpha_beta_root(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  moves: &Vec<Move>,
  depth: u8,
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, ply_count(board));
  let mut alpha = Score::Loss(0);
  let mut best_pv = Vec::new();
  for candidate in moves {
    if let Some(position) = board.move_if_legal(*candidate) {
      let (mut pv, mut score) = alpha_beta(
        state,
        settings,
        &position,
        depth - 1,
        Score::Loss(0),
        -alpha,
      );
      if settings.search_is_over() {
        return (best_pv, alpha);
      }
      score = -score;
      if score > alpha {
        alpha = score;
        let mut new_pv = vec![*candidate];
        new_pv.append(&mut pv);
        best_pv = new_pv;
      } else if score == Score::Loss(0) && !settings.search_is_over() {
        eprintln!(
          "Position {} move {} bug found without unwind",
          board.to_string(),
          candidate.to_string()
        )
      }
    }
  }
  (best_pv, alpha)
}

/// Search the specified position and moves to the specified depth
pub fn search(
  state: &mut State,
  mut settings: SearchConfig,
  position: &Board,
  mut moves: Vec<Move>,
  mut out: Option<Stdout>,
) -> Vec<Move> {
  state.table.new_position(position);
  let mut best_pv = vec![moves[0]];
  let mut current_score = evaluate(position);
  let mut depth = 0;
  while depth < settings.max_depth {
    depth += 1;
    let (pv, score) = alpha_beta_root(state, &mut settings, position, &moves, depth);
    let time = settings.start.elapsed().as_millis();
    let nps = (1000 * settings.nodes) / max(time as usize, 1);
    if !pv.is_empty() {
      best_pv = pv;
      current_score = score;
    }
    if let Some(ref mut out) = out {
      out
        .write(
          format!(
            "info depth {depth} seldepth {} score {} time {time} nodes {} nps {nps} hashfull {} pv {}\n",
            settings.seldepth - ply_count(position),
            current_score.show_uci(position.moves()),
            settings.nodes,
            state.table.capacity(),
            best_pv
              .iter()
              .map(Move::to_string)
              .collect::<Vec<String>>()
              .join(" ")
          )
          .as_bytes(),
        )
        .ok();
    }
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
