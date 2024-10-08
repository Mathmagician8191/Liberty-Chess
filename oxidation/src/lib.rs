#![warn(missing_docs, unused)]
//! A chess engine for Liberty Chess

use crate::evaluate::evaluate;
use crate::history::History;
use crate::movepicker::MovePicker;
use crate::parameters::Parameters;
use crate::search::alpha_beta_root;
use crate::search::SearchParameters;
use crate::tt::TranspositionTable;
use liberty_chess::moves::Move;
use liberty_chess::{perft, Board, ExtraFlags, Piece, PAWN};
use parameters::DEFAULT_PARAMETERS;
use parameters::PAWN_SCALING_NUMERATOR;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::{max, Ordering};
use std::io::{Stdout, Write};
use std::ops::Mul;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Instant;
use ulci::client::Message;
use ulci::server::UlciResult;
use ulci::{AnalysisResult, Score, SearchTime};

#[cfg(not(feature = "feature_extraction"))]
use crate::parameters::PackedParameters;

/// Evaluation
pub mod evaluate;
/// Interface for efficiently integrating into another application
pub mod glue;
/// Tunable parameters
pub mod parameters;
/// Searching through a position
pub mod search;

mod history;
mod movepicker;
mod tt;

/// The version number of the engine
pub const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");

/// Default Hash size
pub const HASH_SIZE: usize = 64;
/// Default Multi-PV lines
pub const MULTI_PV_COUNT: u16 = 1;

const DRAW_SCORE: Score = Score::Centipawn(0);

/// The output type to use for analysis results
pub enum Output<'a> {
  /// Output to the provided stdout
  String(Stdout),
  /// Output to the provided results channel
  Channel(&'a Sender<UlciResult>),
}

struct StackEntry {
  movepicker: MovePicker,
  board: Board,
  eval: Option<i32>,
}

impl StackEntry {
  fn new(board: Board) -> Self {
    Self {
      movepicker: MovePicker::new(),
      board,
      eval: None,
    }
  }

  fn pick_move(&mut self, history: &History, parameters: &Parameters<i32>) -> Option<(Move, bool)> {
    self.movepicker.pick_move(history, parameters, &self.board)
  }
}

/// The state of the engine
pub struct State {
  /// A cache of previously visited positions
  pub table: TranspositionTable,
  // Also stores countermoves
  history: History,
  // Thing indexed by ply, contains heap allocation caches
  stack: Vec<StackEntry>,
  search_parameters: SearchParameters,
  parameters: Parameters<i32>,
  #[cfg(not(feature = "feature_extraction"))]
  packed_parameters: PackedParameters,
  promotion_values: (i32, i32),
}

impl State {
  /// Initialise a new state, sets up a TT of the provided capacity
  #[must_use]
  pub fn new(
    megabytes: usize,
    position: &Board,
    search_parameters: SearchParameters,
    parameters: Parameters<i32>,
  ) -> Self {
    let promotion_values = get_promotion_values(position.promotion_options(), &parameters);
    Self {
      table: TranspositionTable::new(megabytes, position),
      history: History::new(position.width(), position.height()),
      stack: Vec::new(),
      search_parameters,
      parameters,
      #[cfg(not(feature = "feature_extraction"))]
      packed_parameters: parameters.into(),
      promotion_values,
    }
  }

  /// Updates the state with the new position
  ///
  /// Returns true if the hash was cleared
  pub fn new_position(&mut self, position: &Board) -> bool {
    self
      .history
      .new_position(position.width(), position.height());
    self.stack.clear();
    self.promotion_values = get_promotion_values(position.promotion_options(), &self.parameters);
    self.table.new_position(position)
  }

  /// Clears the hash
  pub fn new_game(&mut self, position: &Board) {
    self.history.clear(position.width(), position.height());
    self.stack.clear();
    self.promotion_values = get_promotion_values(position.promotion_options(), &self.parameters);
    self.table.clear(ExtraFlags::new(position));
  }

  /// Set up the stack to analyse a position
  pub fn set_first_stack_entry(&mut self, board: &Board) {
    self.stack.clear();
    self.stack.push(StackEntry::new(board.clone()));
  }
}

/// Convert promotion options to values
///
/// For evaluating the advanced pawn bonus
pub fn get_promotion_values<T: Copy + PartialOrd + Mul<T, Output = T> + From<i32>>(
  promotions: &[Piece],
  parameters: &Parameters<T>,
) -> (T, T) {
  let piece = promotions
    .iter()
    .max_by(|p, q| {
      let p = parameters.pieces[usize::from(p.unsigned_abs()) - 1].1;
      let q = parameters.pieces[usize::from(q.unsigned_abs()) - 1].1;
      p.partial_cmp(&q).unwrap_or(Ordering::Equal)
    })
    .unwrap_or(&PAWN);
  let pieces = parameters.pieces[usize::from(piece.unsigned_abs()) - 1];
  let scale_factor = T::from(PAWN_SCALING_NUMERATOR);
  (pieces.0 * scale_factor, pieces.1 * scale_factor)
}

/// Configuration for the search
pub struct SearchConfig<'a> {
  start: Instant,
  max_depth: u8,
  max_time: u128,
  max_nodes: usize,
  initial_alpha: Score,
  hard_tm: bool,
  rx: &'a Receiver<Message>,
  stopped: bool,
  nodes: usize,
  debug: &'a mut bool,
  // maximum ply count reached
  seldepth: usize,
  millis: u128,
  // variables to track when to check the time
  last_ms_nodes: usize,
  check_frequency: usize,
  next_check: usize,
  // nodetm state
  best_move_nodes: usize,
}

impl<'a> SearchConfig<'a> {
  /// Initialise the search config
  fn new(
    max_depth: u8,
    max_time: u128,
    max_nodes: usize,
    initial_alpha: Score,
    hard_tm: bool,
    rx: &'a Receiver<Message>,
    debug: &'a mut bool,
  ) -> Self {
    Self {
      start: Instant::now(),
      max_depth,
      max_time,
      max_nodes,
      initial_alpha,
      hard_tm,
      rx,
      stopped: false,
      nodes: 0,
      debug,
      seldepth: 0,
      millis: 0,
      last_ms_nodes: 0,
      check_frequency: 1,
      next_check: 1,
      best_move_nodes: 0,
    }
  }

  /// Initialise the search config based on the search time
  pub fn new_time(
    board: &Board,
    time: SearchTime,
    rx: &'a Receiver<Message>,
    debug: &'a mut bool,
  ) -> Self {
    match time {
      SearchTime::Increment(time, inc) => {
        let time = time.saturating_sub(100);
        let time = time.min(time / 15 + 3 * inc / 4);
        let time = 1.max(time);
        Self::new(u8::MAX, time, usize::MAX, Score::Loss(0), false, rx, debug)
      }
      SearchTime::Asymmetric(wtime, winc, btime, binc) => {
        let (time, inc) = if board.to_move() {
          (wtime, winc)
        } else {
          (btime, binc)
        };
        let time = time.saturating_sub(100);
        let time = time.min(time / 15 + 3 * inc / 4);
        let time = 1.max(time);
        Self::new(u8::MAX, time, usize::MAX, Score::Loss(0), false, rx, debug)
      }
      SearchTime::Infinite => Self::new(
        u8::MAX,
        u128::MAX,
        usize::MAX,
        Score::Loss(0),
        true,
        rx,
        debug,
      ),
      SearchTime::Other(limits) => Self::new(
        limits.depth,
        limits.time,
        limits.nodes,
        Score::Loss(0),
        true,
        rx,
        debug,
      ),
      SearchTime::Mate(moves) => Self::new(
        u8::MAX,
        u128::MAX,
        usize::MAX,
        Score::Win(moves + board.moves() + 1),
        true,
        rx,
        debug,
      ),
    }
  }

  fn search_is_over(&mut self) -> bool {
    if self.stopped || self.nodes >= self.max_nodes {
      self.stopped = true;
      return true;
    }
    if self.nodes >= self.next_check {
      let millis = self.start.elapsed().as_millis();
      if millis > self.millis {
        self.millis = millis;
        if millis >= self.max_time {
          self.stopped = true;
          return true;
        }
        loop {
          match self.rx.try_recv() {
            Ok(message) => match message {
              Message::SetDebug(new_debug) => *self.debug = new_debug,
              Message::UpdatePosition(_) => {
                println!("info error search in progress, cannot change position")
              }
              Message::Go(_)
              | Message::Eval
              | Message::Bench(_)
              | Message::NewGame
              | Message::Perft(_) => {
                println!("info error already searching, cannot start new search")
              }
              Message::Stop => {
                self.stopped = true;
                return true;
              }
              Message::UpdateOption(..) => {
                println!("info error cannot change options during search")
              }
              Message::IsReady => println!("readyok"),
              Message::Clock(_) | Message::Info(_) => (),
            },
            Err(TryRecvError::Disconnected) => {
              self.stopped = true;
              return true;
            }
            Err(TryRecvError::Empty) => break,
          }
        }
        let elapsed_nodes = self.nodes - self.last_ms_nodes;
        self.last_ms_nodes = self.nodes;
        self.check_frequency = elapsed_nodes / 2;
      }
      self.next_check = self.nodes + self.check_frequency;
    }
    false
  }

  fn soft_limit(&self, multipv: bool) -> u128 {
    if multipv {
      self.max_time / 3
    } else {
      let best_move_permill = if self.nodes == 0 {
        0
      } else {
        (self.best_move_nodes * 1000 / self.nodes) as u128
      };
      self.max_time * (1410 - best_move_permill) / 2282
    }
  }
}

/// Returns a random legal move from the provided position, if one exists
#[must_use]
pub fn random_move(board: &Board) -> Option<Move> {
  let moves = board.generate_legal();
  moves.choose(&mut thread_rng())?.last_move
}

/// Returns the top capture by MVV-LVA or a random quiet if there are no captures
#[must_use]
pub fn mvvlva_move(board: &Board) -> Option<Move> {
  let (captures, quiets) = get_move_order(&DEFAULT_PARAMETERS, board, &[]);
  if let Some(capture) = captures.first() {
    Some(*capture)
  } else {
    quiets.choose(&mut thread_rng()).copied()
  }
}

/// Sort the searchmoves from a position
#[must_use]
pub fn get_move_order(
  parameters: &Parameters<i32>,
  position: &Board,
  searchmoves: &[Move],
) -> (Vec<Move>, Vec<Move>) {
  let mut captures = Vec::new();
  let mut quiets = Vec::new();
  position.generate_pseudolegal(&mut captures, &mut quiets);
  let (mut captures, quiets): (Vec<(Move, u8, u8)>, Vec<Move>) = if searchmoves.is_empty() {
    (
      captures
        .into_iter()
        .filter(|(m, _, _)| position.move_if_legal(*m).is_some())
        .collect(),
      quiets
        .into_iter()
        .filter(|m| position.move_if_legal(*m).is_some())
        .collect(),
    )
  } else {
    (
      captures
        .into_iter()
        .filter(|(m, _, _)| searchmoves.contains(m) && position.move_if_legal(*m).is_some())
        .collect(),
      quiets
        .into_iter()
        .filter(|m| searchmoves.contains(m) && position.move_if_legal(*m).is_some())
        .collect(),
    )
  };
  captures.sort_by_key(|(_, piece, capture)| {
    parameters.pieces[usize::from(*piece - 1)].0
      - 100 * parameters.pieces[usize::from(*capture - 1)].0
  });
  let captures: Vec<Move> = captures.into_iter().map(|(m, _, _)| m).collect();
  (captures, quiets)
}

fn print_info(
  out: &mut Output,
  position: &Board,
  score: Score,
  depth: u8,
  settings: &SearchConfig,
  pv: &[Move],
  pv_line: u16,
  show_pv_line: bool,
  hashfull: usize,
) {
  let time = settings.start.elapsed().as_millis();
  let nps = (1000 * settings.nodes) / max(time as usize, 1);
  match out {
    Output::String(ref mut out) => {
      let multipv = if show_pv_line {
        format!("multipv {pv_line} ")
      } else {
        String::new()
      };
      out
        .write_all(
          format!(
            "info depth {depth} seldepth {} score {} time {time} nodes {} nps {nps} hashfull {hashfull} {multipv}pv {}\n",
            settings.seldepth,
            score.show_uci(position.moves(), position.to_move()),
            settings.nodes,
            pv
              .iter()
              .map(Move::to_string)
              .collect::<Vec<String>>()
              .join(" ")
          )
          .as_bytes(),
        )
        .ok();
    }
    Output::Channel(tx) => {
      tx.send(UlciResult::Analysis(AnalysisResult {
        pv: pv.to_vec(),
        score,
        depth: u16::from(depth),
        nodes: settings.nodes,
        time,
        wdl: None,
        pv_line,
      }))
      .ok();
    }
  }
}

/// Search the specified position and moves to the specified depth
pub fn search(
  state: &mut State,
  settings: &mut SearchConfig,
  position: &mut Board,
  searchmoves: &[Move],
  multipv: u16,
  mut out: Output,
) -> Vec<Move> {
  position.skip_checkmate = true;
  let mut current_score = Score::Centipawn(evaluate(state, position));
  let mut depth = 0;
  let mut display_depth = 0;
  let (captures, mut quiets) = get_move_order(&state.parameters, position, searchmoves);
  let mut best_moves = Vec::new();
  if let (_, Some(ttmove)) = state.table.get(
    position.hash(),
    position.moves(),
    settings.initial_alpha,
    Score::Win(0),
    depth,
  ) {
    if searchmoves.is_empty() || searchmoves.contains(&ttmove) {
      best_moves.push(ttmove);
    }
  };
  let moves = captures.len() + quiets.len();
  let mut best_pv = if let Some(best_move) = best_moves.first() {
    vec![*best_move]
  } else if let Some(capture) = captures.first() {
    vec![*capture]
  } else if let Some(quiet) = quiets.first() {
    vec![*quiet]
  } else {
    Vec::new()
  };
  'outer: while depth < settings.max_depth
    && (settings.hard_tm
      || settings.start.elapsed().as_millis() <= settings.soft_limit(multipv > 1))
  {
    depth += 1;
    let mut excluded_moves = Vec::new();
    for pv_line in 1..=multipv {
      settings.seldepth = 0;
      let (pv, score) = alpha_beta_root(
        state,
        settings,
        position,
        &captures,
        &mut quiets,
        searchmoves.is_empty() && pv_line == 1,
        &best_moves,
        &excluded_moves,
        depth,
        pv_line,
        multipv > 1,
        &mut out,
      );
      if !pv.is_empty() {
        display_depth = depth;
        if let Some(best_move) = pv.first() {
          excluded_moves.push(*best_move);
        }
        if pv_line == 1 {
          best_pv.clone_from(&pv);
        }
        current_score = score;
      } else if !settings.search_is_over() {
        display_depth = depth;
        if pv_line > 1 {
          break;
        }
      }
      print_info(
        &mut out,
        position,
        current_score,
        display_depth,
        settings,
        &pv,
        pv_line,
        multipv > 1,
        state.table.capacity(),
      );
      if settings.search_is_over() {
        break 'outer;
      }
    }
    if !settings.hard_tm && moves <= 1 {
      break;
    }
    best_moves = excluded_moves;
  }
  best_pv
}

/// Search the specified position to a certain depth and return the node count
pub fn bench(
  state: &mut State,
  board: &mut Board,
  depth: u8,
  debug: &mut bool,
  rx: &Receiver<Message>,
  out: Output,
) -> usize {
  println!("Bench for position {}", board.to_string());
  board.skip_checkmate = true;
  state.new_game(board);
  let mut settings = SearchConfig::new(
    depth,
    u128::MAX,
    usize::MAX,
    Score::Loss(0),
    true,
    rx,
    debug,
  );
  search(state, &mut settings, board, &[], 1, out);
  // calculate branching factor
  let log_nodes = (settings.nodes as f64).ln();
  let nodes_per_depth = log_nodes / f64::from(depth);
  println!("Branching factor: {:.3}", nodes_per_depth.exp());
  settings.nodes
}

/// Run perft on the specified position
pub fn divide(board: &Board, depth: usize) {
  let mut board = board.clone();
  board.skip_checkmate = true;
  let start = Instant::now();
  let mut total = 0;
  for position in board.generate_legal() {
    let subtotal = perft(&position, depth - 1);
    total += subtotal;
    println!(
      "{}: {subtotal}",
      position
        .last_move
        .map_or("0000".to_owned(), |m| m.to_string())
    );
  }
  println!("Nodes searched: {total}");
  println!("Elapsed time: {}ms", start.elapsed().as_millis());
}
