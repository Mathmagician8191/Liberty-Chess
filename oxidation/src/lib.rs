#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A chess engine for Liberty Chess

use history::History;
use liberty_chess::moves::Move;
use liberty_chess::{Board, Gamestate};
use parameters::{
  Parameters, DEFAULT_PARAMETERS, EDGE_DISTANCE, ENDGAME_FACTOR, ENDGAME_THRESHOLD,
  MIDDLEGAME_PIECE_VALUES,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::{max, min};
use std::io::{Stdout, Write};
use std::sync::mpsc::TryRecvError::Disconnected;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use tt::{Entry, ScoreType, TranspositionTable};
use ulci::client::Message;
use ulci::server::{AnalysisResult, UlciResult};
use ulci::{OptionValue, Score, SearchTime};

/// Interface for efficiently integrating into another application
pub mod glue;
/// Tunable parameters
pub mod parameters;

mod history;
mod tt;

/// The version number of the engine
pub const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");

/// Default Quiescence depth
pub const QDEPTH: u8 = 5;
/// Default Hash size
pub const HASH_SIZE: usize = 64;

/// Internal naming thing - do not use
///
/// Public due to being required in the binary
pub const QDEPTH_NAME: &str = "QDepth";
/// Internal naming thing - do not use
///
/// Public due to being required in the binary
pub const HASH_NAME: &str = "Hash";

/// The output type to use for analysis results
pub enum Output<'a> {
  /// Output to the provided stdout
  String(Stdout),
  /// Output to the provided results channel
  Channel(&'a Sender<UlciResult>),
}

/// The state of the engine
pub struct State {
  table: TranspositionTable,
  history: History,
}

impl State {
  /// Initialise a new state, sets up a TT of the provided capacity
  #[must_use]
  pub fn new(megabytes: usize, position: &Board) -> Self {
    Self {
      table: TranspositionTable::new(megabytes),
      history: History::new(position.width(), position.height()),
    }
  }

  /// Updates the state with the new position
  ///
  /// Returns true if the hash was cleared
  pub fn new_position(&mut self, position: &Board) -> bool {
    self.history.clear(position.width(), position.height());
    self.table.new_position(position)
  }

  /// Clears the hash
  pub fn new_game(&mut self, position: &Board) {
    self.history.clear(position.width(), position.height());
    self.table.clear();
  }
}

/// Configuration for the search
pub struct SearchConfig<'a> {
  qdepth: &'a mut u8,
  start: Instant,
  max_depth: u8,
  max_time: u128,
  max_nodes: usize,
  hard_tm: bool,
  rx: &'a Receiver<Message>,
  stopped: bool,
  nodes: usize,
  debug: &'a mut bool,
  // maximum ply count reached
  seldepth: u32,
  millis: u128,
}

impl<'a> SearchConfig<'a> {
  /// Initialise the search config
  fn new(
    qdepth: &'a mut u8,
    max_depth: u8,
    max_time: u128,
    max_nodes: usize,
    hard_tm: bool,
    rx: &'a Receiver<Message>,
    debug: &'a mut bool,
  ) -> Self {
    Self {
      qdepth,
      start: Instant::now(),
      max_depth,
      max_time,
      max_nodes,
      hard_tm,
      rx,
      stopped: false,
      nodes: 0,
      debug,
      seldepth: 0,
      millis: 0,
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
        Self::new(qdepth, u8::MAX, time, usize::MAX, false, rx, debug)
      }
      SearchTime::Infinite => Self::new(qdepth, u8::MAX, u128::MAX, usize::MAX, true, rx, debug),
      SearchTime::Other(limits) => Self::new(
        qdepth,
        limits.depth,
        limits.time,
        limits.nodes,
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
    let millis = self.start.elapsed().as_millis();
    if millis > self.millis {
      self.millis = millis;
      if millis >= self.max_time {
        self.stopped = true;
        return true;
      }
      for message in self.rx.try_iter() {
        match message {
          Message::SetDebug(new_debug) => *self.debug = new_debug,
          Message::UpdatePosition(_) => {
            if *self.debug {
              println!("info string servererror search in progress");
            }
          }
          Message::Go(_) | Message::Eval | Message::Bench(_) | Message::NewGame => {
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
pub fn get_move_order(state: &State, position: &Board, searchmoves: &Vec<Move>) -> Vec<Move> {
  let (captures, other) = position.generate_pseudolegal();
  let (mut captures, mut other): (Vec<(Move, u8, u8)>, Vec<Move>) = if searchmoves.is_empty() {
    (
      captures
        .into_iter()
        .filter(|(m, _, _)| position.move_if_legal(*m).is_some())
        .collect(),
      other
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
      other
        .into_iter()
        .filter(|m| searchmoves.contains(m) && position.move_if_legal(*m).is_some())
        .collect(),
    )
  };
  captures.shuffle(&mut thread_rng());
  captures.sort_by_key(|(_, piece, capture)| {
    MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
      - MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
  });
  other.shuffle(&mut thread_rng());
  other.sort_by_key(|r#move| {
    u32::MAX
      - state.history.get(
        position.to_move(),
        position.get_piece(r#move.start()).unsigned_abs(),
        r#move.end(),
      )
  });
  let mut moves: Vec<Move> = captures.into_iter().map(|(m, _, _)| m).collect();
  moves.append(&mut other);
  moves
}

#[must_use]
fn ply_count(board: &Board) -> u32 {
  board.moves() * 2 + u32::from(!board.to_move())
}

/// Returns the static evaluation of the provided position
#[must_use]
pub fn evaluate(board: &Board, parameters: &Parameters) -> Score {
  match board.state() {
    Gamestate::InProgress => {
      let middlegame = evaluate_middlegame(
        board,
        &parameters.middlegame_pieces,
        &parameters.middlegame_edge,
      );
      let endgame = evaluate_endgame(board, &parameters.endgame_pieces, &parameters.endgame_edge);
      let mut material = 0;
      for piece in board.board().as_row_major() {
        if piece != 0 {
          let piece_type = piece.unsigned_abs() as usize - 1;
          material += ENDGAME_FACTOR[piece_type];
        }
      }
      material = min(material, ENDGAME_THRESHOLD);
      let score = material * middlegame + (ENDGAME_THRESHOLD - material) * endgame;
      Score::Centipawn(score / ENDGAME_THRESHOLD)
    }
    Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
      Score::Centipawn(0)
    }
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(board.moves()),
  }
}

#[must_use]
fn evaluate_middlegame(
  board: &Board,
  piece_values: &[i32; 18],
  edge_avoidance: &[[i32; EDGE_DISTANCE]; 18],
) -> i32 {
  let mut score = 0;
  let pieces = board.board();
  for i in 0..board.height() {
    for j in 0..board.width() {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let multiplier = if piece > 0 { 1 } else { -1 };
        let piece_type = piece.unsigned_abs() as usize - 1;
        let mut value = piece_values[piece_type];
        let horizontal_distance = min(i, board.height() - 1 - i);
        if horizontal_distance < EDGE_DISTANCE {
          value -= edge_avoidance[piece_type][horizontal_distance];
        }
        let vertical_distance = min(j, board.width() - 1 - j);
        if vertical_distance < EDGE_DISTANCE {
          value -= edge_avoidance[piece_type][vertical_distance];
        }
        score += value * multiplier;
      }
    }
  }
  if !board.to_move() {
    score *= -1;
  }
  score
}

#[must_use]
fn evaluate_endgame(
  board: &Board,
  piece_values: &[i32; 18],
  edge_avoidance: &[[i32; EDGE_DISTANCE]; 18],
) -> i32 {
  let mut score = 0;
  let pieces = board.board();
  for i in 0..board.height() {
    for j in 0..board.width() {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let multiplier = if piece > 0 { 1 } else { -1 };
        let piece_type = piece.unsigned_abs() as usize - 1;
        let mut value = piece_values[piece_type];
        let horizontal_distance = min(i, board.height() - 1 - i);
        if horizontal_distance < EDGE_DISTANCE {
          value -= edge_avoidance[piece_type][horizontal_distance];
        }
        let vertical_distance = min(j, board.width() - 1 - j);
        if vertical_distance < EDGE_DISTANCE {
          value -= edge_avoidance[piece_type][vertical_distance];
        }
        score += value * multiplier;
      }
    }
  }
  if !board.to_move() {
    score *= -1;
  }
  score
}

/// Run a quiescence search of the given position
pub fn quiescence(
  state: &State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  mut alpha: Score,
  beta: Score,
  parameters: &Parameters,
) -> (Vec<Move>, Score) {
  let hash = board.hash();
  if board.state() == Gamestate::InProgress {
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, 0);
    if let Some(score) = score {
      let mut pv = Vec::new();
      if let Some(bestmove) = ttmove {
        pv.push(bestmove);
      }
      return (pv, score);
    }
  }
  let score = evaluate(board, parameters);
  settings.nodes += 1;
  settings.seldepth = max(settings.seldepth, ply_count(board));
  if score >= beta {
    return (Vec::new(), beta);
  }
  let mut best_pv = Vec::new();
  if score > alpha {
    alpha = score;
  }
  if !settings.search_is_over() && (depth != 0) && (board.state() == Gamestate::InProgress) {
    let mut moves = board.generate_qsearch();
    moves.sort_by_key(|(_, piece, capture)| {
      parameters.middlegame_pieces[usize::from(*piece - 1)]
        - parameters.middlegame_pieces[usize::from(*capture - 1)]
    });
    for (bestmove, _, _) in moves {
      if let Some(position) = board.test_move_legality(bestmove) {
        let (mut pv, mut score) = quiescence(
          state,
          settings,
          &position,
          depth - 1,
          -beta,
          -alpha,
          parameters,
        );
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
  // not allowed to nullmove if previous nullmove
  nullmove: bool,
) -> (Vec<Move>, Score) {
  if let Score::Win(movecount) = alpha {
    let moves = board.moves();
    if moves >= movecount {
      // Mate distance pruning
      return (Vec::new(), alpha);
    }
  }
  if board.in_check() {
    depth += 1;
  }
  if board.state() != Gamestate::InProgress {
    quiescence(
      state,
      settings,
      board,
      *settings.qdepth,
      alpha,
      beta,
      &DEFAULT_PARAMETERS,
    )
  } else if depth == 0 {
    let (pv, score) = quiescence(
      state,
      settings,
      board,
      *settings.qdepth,
      alpha,
      beta,
      &DEFAULT_PARAMETERS,
    );
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
    let tt_depth = depth;
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, tt_depth);
    if let Some(score) = score {
      let mut pv = Vec::new();
      if let Some(bestmove) = ttmove {
        pv.push(bestmove);
      }
      return (pv, score);
    }
    if !nullmove && depth > 2 && board.has_pieces() && evaluate(board, &DEFAULT_PARAMETERS) >= beta
    {
      if let Some(nullmove) = board.nullmove() {
        let mut score = zero_window_search(state, settings, &nullmove, depth - 2, -beta, true);
        score = -score;
        if score >= beta {
          // Null move reduction
          depth -= 2;
        }
      }
    }
    let mut best_pv = Vec::new();
    // Handle TTmove
    if let Some(ttmove) = ttmove {
      if let Some(position) = board.test_move_legality(ttmove) {
        let (mut pv, mut score) =
          alpha_beta(state, settings, &position, depth - 1, -beta, -alpha, false);
        score = -score;
        if score >= beta {
          state.history.store(
            board.to_move(),
            board.get_piece(ttmove.start()).unsigned_abs(),
            ttmove.end(),
            depth,
          );
          state.table.store(Entry {
            hash,
            depth: tt_depth,
            movecount: board.moves(),
            scoretype: ScoreType::LowerBound,
            score: beta,
            bestmove: Some(ttmove),
          });
          return (Vec::new(), beta);
        }
        if score > alpha {
          alpha = score;
          let mut new_pv = vec![ttmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
        if settings.search_is_over() {
          if !best_pv.is_empty() {
            state.table.store(Entry {
              hash,
              depth: tt_depth,
              movecount: board.moves(),
              scoretype: ScoreType::LowerBound,
              score: alpha,
              bestmove: Some(ttmove),
            });
          }
          return (best_pv, alpha);
        }
      }
    }
    let (mut moves, mut other) = board.generate_pseudolegal();
    moves.sort_by_key(|(_, piece, capture)| {
      MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
        - MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
    });
    let mut moves: Vec<Move> = moves.into_iter().map(|(m, _, _)| m).collect();
    other.sort_by_key(|r#move| {
      u32::MAX
        - state.history.get(
          board.to_move(),
          board.get_piece(r#move.start()).unsigned_abs(),
          r#move.end(),
        )
    });
    moves.append(&mut other);
    for bestmove in moves {
      if Some(bestmove) != ttmove {
        if let Some(position) = board.test_move_legality(bestmove) {
          let (mut pv, mut score) =
            alpha_beta(state, settings, &position, depth - 1, -beta, -alpha, false);
          score = -score;
          if score >= beta {
            state.history.store(
              board.to_move(),
              board.get_piece(bestmove.start()).unsigned_abs(),
              bestmove.end(),
              depth,
            );
            state.table.store(Entry {
              hash,
              depth: tt_depth,
              movecount: board.moves(),
              scoretype: ScoreType::LowerBound,
              score: beta,
              bestmove: Some(bestmove),
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
                depth: tt_depth,
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
    }
    let scoretype = if best_pv.is_empty() {
      ScoreType::UpperBound
    } else {
      ScoreType::Exact
    };
    state.table.store(Entry {
      hash,
      depth: tt_depth,
      movecount: board.moves(),
      scoretype,
      score: alpha,
      bestmove: best_pv.get(0).copied(),
    });
    (best_pv, alpha)
  }
}

fn zero_window_search(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  alpha: Score,
  // not allowed to nullmove if previous nullmove
  nullmove: bool,
) -> Score {
  let beta = match alpha {
    Score::Centipawn(cp) => Score::Centipawn(cp + 1),
    Score::Win(moves) => Score::Win(moves - 1),
    Score::Loss(moves) => Score::Loss(moves + 1),
  };
  let (_, score) = alpha_beta(state, settings, board, depth, alpha, beta, nullmove);
  score
}

fn alpha_beta_root(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  moves: &Vec<Move>,
  depth: u8,
) -> (Vec<Move>, Score) {
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
        false,
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
  mut out: Output,
) -> Vec<Move> {
  let mut best_pv = vec![moves[0]];
  let mut current_score = evaluate(position, &DEFAULT_PARAMETERS);
  let mut depth = 0;
  while depth < settings.max_depth
    && (settings.hard_tm || settings.start.elapsed().as_millis() * 2 <= settings.max_time)
  {
    depth += 1;
    let (pv, score) = alpha_beta_root(state, &mut settings, position, &moves, depth);
    let time = settings.start.elapsed().as_millis();
    let nps = (1000 * settings.nodes) / max(time as usize, 1);
    if !pv.is_empty() {
      best_pv = pv;
      current_score = score;
    }
    match out {
      Output::String(ref mut out) => {
        out
          .write(
            format!(
              "info depth {depth} seldepth {} score {} time {time} nodes {} nps {nps} hashfull {} pv {}\n",
              settings.seldepth - ply_count(position),
              current_score.show_uci(position.moves(), position.to_move()),
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
      Output::Channel(tx) => {
        tx.send(UlciResult::Analysis(AnalysisResult {
          pv: best_pv.clone(),
          score: current_score,
          depth: u16::from(depth),
          nodes: settings.nodes,
          time,
          wdl: None,
        }))
        .ok();
      }
    }
    if settings.search_is_over() {
      break;
    }
    let bestmove = best_pv[0];
    let mut sorted_moves = vec![bestmove];
    sorted_moves.append(
      &mut get_move_order(state, position, &moves)
        .into_iter()
        .filter(|m| *m != bestmove)
        .collect::<Vec<Move>>(),
    );
    moves = sorted_moves;
  }
  best_pv
}

/// Search the specified position to a certain depth and return the node count
pub fn bench(
  board: &Board,
  depth: u8,
  qdepth: &mut u8,
  debug: &mut bool,
  hash: usize,
  rx: &Receiver<Message>,
  mut out: Output,
) -> usize {
  println!("Bench for position {}", board.to_string());
  let mut settings = SearchConfig::new(qdepth, depth, u128::MAX, usize::MAX, true, rx, debug);
  let mut state = State::new(hash, board);
  let (mut moves, mut other) = board.generate_pseudolegal();
  moves.sort_by_key(|(_, piece, capture)| {
    MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
      - MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
  });
  let mut moves: Vec<Move> = moves.into_iter().map(|(m, _, _)| m).collect();
  moves.append(&mut other);
  let mut best_pv = vec![moves[0]];
  let mut current_score = evaluate(board, &DEFAULT_PARAMETERS);
  let mut depth = 0;
  while depth < settings.max_depth
    && (settings.start.elapsed().as_millis() * 2 <= settings.max_time)
  {
    depth += 1;
    let (pv, score) = alpha_beta_root(&mut state, &mut settings, board, &moves, depth);
    let time = settings.start.elapsed().as_millis();
    let nps = (1000 * settings.nodes) / max(time as usize, 1);
    if !pv.is_empty() {
      best_pv = pv;
      current_score = score;
    }
    match out {
      Output::String(ref mut out) => {
        out
          .write(
            format!(
              "info depth {depth} seldepth {} score {} time {time} nodes {} nps {nps} hashfull {} pv {}\n",
              settings.seldepth - ply_count(board),
              current_score.show_uci(board.moves(), board.to_move()),
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
      Output::Channel(tx) => {
        tx.send(UlciResult::Analysis(AnalysisResult {
          pv: best_pv.clone(),
          score: current_score,
          depth: u16::from(depth),
          nodes: settings.nodes,
          time,
          wdl: None,
        }))
        .ok();
      }
    }
    if settings.search_is_over() {
      break;
    }
    let bestmove = best_pv[0];
    let (mut new_moves, mut other) = board.generate_pseudolegal();
    new_moves.sort_by_key(|(_, piece, capture)| {
      MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
        - MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
    });
    let mut new_moves: Vec<Move> = new_moves.into_iter().map(|(m, _, _)| m).collect();
    other.sort_by_key(|r#move| {
      u32::MAX
        - state.history.get(
          board.to_move(),
          board.get_piece(r#move.start()).unsigned_abs(),
          r#move.end(),
        )
    });
    new_moves.append(&mut other);
    let mut sorted_moves = vec![bestmove];
    sorted_moves.append(
      &mut new_moves
        .into_iter()
        .filter(|m| *m != bestmove)
        .collect::<Vec<Move>>(),
    );
    moves = sorted_moves;
  }
  // calculate branching factor
  let log_nodes = (settings.nodes as f64).ln();
  let nodes_per_depth = log_nodes / f64::from(depth);
  println!("Branching factor: {:.3}", nodes_per_depth.exp());
  settings.nodes
}
