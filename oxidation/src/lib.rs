#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A chess engine for Liberty Chess

use crate::parameters::{
  Parameters, EDGE_DISTANCE, ENDGAME_FACTOR, ENDGAME_THRESHOLD, HALFMOVE_SCALING,
  MIDDLEGAME_PIECE_VALUES, MIN_HALFMOVES,
};
use array2d::Array2D;
use history::History;
use liberty_chess::moves::Move;
use liberty_chess::{perft, Board, ExtraFlags, Gamestate, Piece, PAWN};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::{max, min};
use std::io::{Stdout, Write};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Instant;
use tt::{Entry, ScoreType, TranspositionTable};
use ulci::client::Message;
use ulci::server::UlciResult;
use ulci::{AnalysisResult, OptionValue, Score, SearchTime};

#[cfg(feature = "pesto")]
use crate::pesto::{EG_PSQT, MG_PSQT};

/// Interface for efficiently integrating into another application
pub mod glue;
/// Tunable parameters
pub mod parameters;

mod history;
mod tt;

#[cfg(feature = "pesto")]
mod pesto;

/// The version number of the engine
pub const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");

/// Default Quiescence depth
pub const QDEPTH: u8 = 4;
/// Default Hash size
pub const HASH_SIZE: usize = 64;

/// Internal naming thing - do not use
///
/// Public due to being required in the binary
pub const QDEPTH_NAME: &str = "QDepth";

const DRAW_SCORE: Score = Score::Centipawn(0);

/// The output type to use for analysis results
pub enum Output<'a> {
  /// Output to the provided stdout
  String(Stdout),
  /// Output to the provided results channel
  Channel(&'a Sender<UlciResult>),
}

/// The state of the engine
pub struct State {
  /// A cache of previously visited positions
  pub table: TranspositionTable,
  history: History,
  parameters: Parameters,
  promotion_values: (i32, i32),
}

impl State {
  /// Initialise a new state, sets up a TT of the provided capacity
  #[must_use]
  pub fn new(megabytes: usize, position: &Board, parameters: Parameters) -> Self {
    let promotion_values = get_promotion_values(position.promotion_options(), &parameters);
    Self {
      table: TranspositionTable::new(megabytes, position),
      history: History::new(position.width(), position.height()),
      parameters,
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
    self.promotion_values = get_promotion_values(position.promotion_options(), &self.parameters);
    self.table.new_position(position)
  }

  /// Clears the hash
  pub fn new_game(&mut self, position: &Board) {
    self.history.clear(position.width(), position.height());
    self.promotion_values = get_promotion_values(position.promotion_options(), &self.parameters);
    self.table.clear(ExtraFlags::new(position));
  }
}

/// Convert promotion options to values
///
/// For the tuner
pub fn get_promotion_values(promotions: &[Piece], parameters: &Parameters) -> (i32, i32) {
  let piece = promotions
    .iter()
    .max_by_key(|p| parameters.eg_pieces[usize::from(p.unsigned_abs()) - 1])
    .unwrap_or(&PAWN);
  (
    parameters.mg_pieces[usize::from(piece.unsigned_abs()) - 1],
    parameters.eg_pieces[usize::from(piece.unsigned_abs()) - 1],
  )
}

/// Configuration for the search
pub struct SearchConfig<'a> {
  qdepth: &'a mut u8,
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
  seldepth: u32,
  millis: u128,
  // variables to track when to check the time
  last_ms_nodes: usize,
  check_frequency: usize,
  next_check: usize,
}

impl<'a> SearchConfig<'a> {
  /// Initialise the search config
  fn new(
    qdepth: &'a mut u8,
    max_depth: u8,
    max_time: u128,
    max_nodes: usize,
    initial_alpha: Score,
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
    }
  }

  /// Initialise the search config based on the search time
  pub fn new_time(
    board: &Board,
    qdepth: &'a mut u8,
    time: SearchTime,
    rx: &'a Receiver<Message>,
    debug: &'a mut bool,
  ) -> Self {
    match time {
      SearchTime::Increment(time, inc) => {
        let time = time.saturating_sub(100);
        let time = time.min(time / 15 + 3 * inc / 4);
        let time = 1.max(time);
        Self::new(
          qdepth,
          u8::MAX,
          time,
          usize::MAX,
          Score::Loss(0),
          false,
          rx,
          debug,
        )
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
        Self::new(
          qdepth,
          u8::MAX,
          time,
          usize::MAX,
          Score::Loss(0),
          false,
          rx,
          debug,
        )
      }
      SearchTime::Infinite => Self::new(
        qdepth,
        u8::MAX,
        u128::MAX,
        usize::MAX,
        Score::Loss(0),
        true,
        rx,
        debug,
      ),
      SearchTime::Other(limits) => Self::new(
        qdepth,
        limits.depth,
        limits.time,
        limits.nodes,
        Score::Loss(0),
        true,
        rx,
        debug,
      ),
      SearchTime::Mate(moves) => Self::new(
        qdepth,
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
            Ok(message) => {
              match message {
                Message::SetDebug(new_debug) => *self.debug = new_debug,
                Message::UpdatePosition(_) => {
                  if *self.debug {
                    println!("info string servererror search in progress");
                  }
                }
                Message::Go(_)
                | Message::Eval
                | Message::Bench(_)
                | Message::NewGame
                | Message::Perft(_) => {
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
                Message::Clock(_) | Message::Info(_) => (),
              }
            }
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
}

/// Returns a random legal move from the provided position, if one exists
#[must_use]
pub fn random_move(board: &Board) -> Option<Move> {
  let moves = board.generate_legal();
  moves.choose(&mut thread_rng())?.last_move
}

/// Sort the searchmoves from a position
#[must_use]
pub fn get_move_order(state: &State, position: &Board, searchmoves: &[Move]) -> Vec<Move> {
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
      - 100 * MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
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

/// Returns the static evaluation from the provided raw data
#[must_use]
pub fn evaluate_raw(
  pieces: &Array2D<Piece>,
  promotion_values: (i32, i32),
  to_move: bool,
  halfmoves: u8,
  parameters: &Parameters,
) -> i32 {
  let mut middlegame = 0;
  let mut endgame = 0;
  let mut material = 0;
  let height = pieces.num_rows();
  let width = pieces.num_columns();
  for i in 0..height {
    for j in 0..width {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let multiplier = if piece > 0 { 1 } else { -1 };
        let piece_type = piece.unsigned_abs() as usize - 1;
        material += ENDGAME_FACTOR[piece_type];
        let mut mg_value = parameters.mg_pieces[piece_type];
        let mut eg_value = parameters.eg_pieces[piece_type];
        #[cfg(feature = "pesto")]
        {
          if height == 8 && width == 8 && piece_type < 6 {
            let index = if piece > 0 { 7 - i } else { i };
            mg_value += MG_PSQT[piece_type][index][j];
            eg_value += EG_PSQT[piece_type][index][j];
          } else {
            let horizontal_distance = min(i, height - 1 - i);
            if horizontal_distance < EDGE_DISTANCE {
              mg_value -= parameters.mg_edge[piece_type][horizontal_distance];
              eg_value -= parameters.eg_edge[piece_type][horizontal_distance];
            }
            let vertical_distance = min(j, width - 1 - j);
            if vertical_distance < EDGE_DISTANCE {
              mg_value -= parameters.mg_edge[piece_type][vertical_distance];
              eg_value -= parameters.eg_edge[piece_type][vertical_distance];
            }
          }
        }
        #[cfg(not(feature = "pesto"))]
        {
          let horizontal_distance = min(i, height - 1 - i);
          if horizontal_distance < EDGE_DISTANCE {
            mg_value -= parameters.mg_edge[piece_type][horizontal_distance];
            eg_value -= parameters.eg_edge[piece_type][horizontal_distance];
          }
          let vertical_distance = min(j, width - 1 - j);
          if vertical_distance < EDGE_DISTANCE {
            mg_value -= parameters.mg_edge[piece_type][vertical_distance];
            eg_value -= parameters.eg_edge[piece_type][vertical_distance];
          }
        }
        if piece.abs() == PAWN {
          // penalty for pawn being blocked
          let block_i = if piece > 0 { i + 1 } else { i - 1 };
          if let Some(piece) = pieces.get(block_i, j) {
            if *piece != 0 {
              let abs_piece = usize::from(piece.unsigned_abs());
              if (*piece > 0) ^ (multiplier > 0) {
                mg_value -= parameters.mg_enemy_pawn_penalty[abs_piece - 1];
                eg_value -= parameters.eg_enemy_pawn_penalty[abs_piece - 1];
              } else {
                mg_value -= parameters.mg_friendly_pawn_penalty[abs_piece - 1];
                eg_value -= parameters.eg_friendly_pawn_penalty[abs_piece - 1];
              }
            }
          }
          // bonus for advanced pawn
          #[cfg(not(feature = "pesto"))]
          {
            let squares_to_go = if piece > 0 { height - 1 - i } else { i } as i32;
            if squares_to_go != 0 {
              mg_value += promotion_values.0
                / (squares_to_go * parameters.pawn_scale_factor + parameters.pawn_scaling_bonus);
              eg_value += promotion_values.1
                / (squares_to_go * parameters.pawn_scale_factor + parameters.pawn_scaling_bonus);
            }
          }
        }
        middlegame += mg_value * multiplier;
        endgame += eg_value * multiplier;
      }
    }
  }
  material = min(material, ENDGAME_THRESHOLD);
  let score = material * middlegame + (ENDGAME_THRESHOLD - material) * endgame;
  let mut score = score / ENDGAME_THRESHOLD;
  if !to_move {
    score *= -1;
  }
  if halfmoves > MIN_HALFMOVES {
    score =
      score * i32::from(HALFMOVE_SCALING + MIN_HALFMOVES) / i32::from(halfmoves + HALFMOVE_SCALING);
  }
  score
}

/// Returns the static evaluation of the provided position
#[must_use]
pub fn evaluate(state: &State, board: &Board) -> Score {
  match board.state() {
    Gamestate::InProgress => {
      let score = evaluate_raw(
        board.board(),
        state.promotion_values,
        board.to_move(),
        board.halfmoves(),
        &state.parameters,
      );
      Score::Centipawn(score)
    }
    Gamestate::Material | Gamestate::FiftyMove | Gamestate::Repetition | Gamestate::Stalemate => {
      DRAW_SCORE
    }
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(board.moves()),
  }
}

/// Run a quiescence search of the given position
fn recaptures(
  state: &State,
  settings: &mut SearchConfig,
  board: &Board,
  mut alpha: Score,
  beta: Score,
  target: (usize, usize),
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, board.ply_count());
  if board.state() == Gamestate::InProgress {
    let best_score = evaluate(state, board);
    if best_score >= beta {
      return (Vec::new(), beta);
    }
    if best_score > alpha {
      alpha = best_score;
    }
    let mut best_pv = Vec::new();
    let mut moves = board.generate_recaptures(target);
    moves.sort_by_key(|(_, piece)| state.parameters.mg_pieces[usize::from(*piece - 1)]);
    for (bestmove, _) in moves {
      if let Some(position) = board.test_move_legality(bestmove) {
        settings.nodes += 1;
        let (mut pv, mut score) = recaptures(state, settings, &position, -beta, -alpha, target);
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
        break;
      }
    }
    (best_pv, alpha)
  } else {
    (Vec::new(), evaluate(state, board))
  }
}

/// Run a quiescence search of the given position
pub fn quiescence(
  state: &State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, board.ply_count());
  if board.state() == Gamestate::InProgress {
    let hash = board.hash();
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, 0);
    if let Some(score) = score {
      let mut pv = Vec::new();
      if let Some(bestmove) = ttmove {
        pv.push(bestmove);
      }
      return (pv, score);
    }
    if depth == 0 {
      return recaptures(
        state,
        settings,
        board,
        alpha,
        beta,
        board.last_move.expect("Board missing last move").end(),
      );
    }
    let score = evaluate(state, board);
    if score >= beta {
      return (Vec::new(), beta);
    }
    if score > alpha {
      alpha = score;
    }
    let mut best_pv = Vec::new();
    if !settings.search_is_over() {
      let mut moves = board.generate_qsearch();
      moves.sort_by_key(|(_, piece, capture)| {
        state.parameters.mg_pieces[usize::from(*piece - 1)]
          - 100 * state.parameters.mg_pieces[usize::from(*capture - 1)]
      });
      for (bestmove, _, _) in moves {
        if let Some(position) = board.test_move_legality(bestmove) {
          settings.nodes += 1;
          let (mut pv, mut score) =
            quiescence(state, settings, &position, depth - 1, -beta, -alpha);
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
  } else {
    (Vec::new(), evaluate(state, board))
  }
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
  let in_check = board.in_check();
  if in_check {
    depth += 1;
  }
  if board.state() != Gamestate::InProgress {
    (Vec::new(), evaluate(state, board))
  } else if depth == 0 {
    let (pv, score) = quiescence(state, settings, board, *settings.qdepth, alpha, beta);
    if !settings.search_is_over() {
      let (tt_flag, tt_score) = if score >= beta {
        (ScoreType::LowerBound, beta)
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
        bestmove: pv.first().copied(),
      });
    }
    (pv, score)
  } else {
    let hash = board.hash();
    let tt_depth = depth;
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, tt_depth);
    let pv_node = match (alpha, beta) {
      (Score::Centipawn(alpha), Score::Centipawn(beta)) => alpha + 1 != beta,
      (Score::Loss(alpha), Score::Loss(beta)) => alpha + 1 != beta,
      (Score::Win(alpha), Score::Win(beta)) => alpha - 1 != beta,
      _ => true,
    };
    if !pv_node {
      if let Some(score) = score {
        let mut pv = Vec::new();
        if let Some(bestmove) = ttmove {
          pv.push(bestmove);
        }
        return (pv, score);
      }
    }
    // Null move pruning
    if !pv_node && !nullmove && depth >= 3 && board.has_pieces() && evaluate(state, board) >= beta {
      if let Some(nullmove) = board.nullmove() {
        let score = -null_move_search(state, settings, &nullmove, depth - 3, -beta);
        if score >= beta {
          // Verification search
          if depth < 4 || zero_window_search(state, settings, board, depth - 3, beta, true) >= beta {
            return (Vec::new(), beta);
          }
        }
      }
    }
    let mut best_pv = Vec::new();
    let mut move_count = 0;
    // Handle TTmove
    if let Some(ttmove) = ttmove {
      if let Some(position) = board.move_if_legal(ttmove) {
        settings.nodes += 1;
        move_count += 1;
        let (mut pv, mut score) = alpha_beta(
          state,
          settings,
          &position,
          depth - 1,
          -beta,
          -alpha,
          nullmove,
        );
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
          return (best_pv, alpha);
        }
      }
    }
    let (mut moves, mut other) = board.generate_pseudolegal();
    moves.sort_by_key(|(_, piece, capture)| {
      MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
        - 100 * MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
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
          settings.nodes += 1;
          move_count += 1;
          let (mut pv, score) = if pv_node && move_count > 1 {
            // Zero window search to see if raises alpha
            let score =
              -zero_window_search(state, settings, &position, depth - 1, -alpha, nullmove);
            if score > alpha {
              let (pv, score) = alpha_beta(
                state,
                settings,
                &position,
                depth - 1,
                -beta,
                -alpha,
                nullmove,
              );
              (pv, -score)
            } else {
              (Vec::new(), score)
            }
          } else {
            let (pv, score) = alpha_beta(
              state,
              settings,
              &position,
              depth - 1,
              -beta,
              -alpha,
              nullmove,
            );
            (pv, -score)
          };
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
            return (best_pv, alpha);
          }
        }
      }
    }
    if move_count == 0 {
      (
        Vec::new(),
        if in_check {
          // Checkmate
          Score::Loss(board.moves())
        } else {
          // Stalemate
          DRAW_SCORE
        },
      )
    } else {
      let (scoretype, bestmove) = if best_pv.is_empty() {
        (ScoreType::UpperBound, ttmove)
      } else {
        (ScoreType::Exact, best_pv.first().copied())
      };
      state.table.store(Entry {
        hash,
        depth: tt_depth,
        movecount: board.moves(),
        scoretype,
        score: alpha,
        bestmove,
      });
      (best_pv, alpha)
    }
  }
}

fn null_move_search(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  alpha: Score,
) -> Score {
  let beta = match alpha {
    Score::Centipawn(cp) => Score::Centipawn(cp + 1),
    Score::Win(moves) => Score::Win(moves - 1),
    Score::Loss(moves) => Score::Loss(moves + 1),
  };
  let (_, score) = alpha_beta(state, settings, board, depth, alpha, beta, true);
  score
}

fn zero_window_search(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  beta: Score,
  nullmove: bool,
) -> Score {
  let alpha = match beta {
    Score::Centipawn(cp) => Score::Centipawn(cp - 1),
    Score::Win(moves) => Score::Win(moves + 1),
    Score::Loss(moves) => Score::Loss(moves - 1),
  };
  let (_, score) = alpha_beta(state, settings, board, depth, alpha, beta, nullmove);
  score
}

fn print_info(
  out: &mut Output,
  position: &Board,
  score: Score,
  depth: u8,
  settings: &SearchConfig,
  pv: &[Move],
  hashfull: usize,
) {
  let time = settings.start.elapsed().as_millis();
  let nps = (1000 * settings.nodes) / max(time as usize, 1);
  match out {
    Output::String(ref mut out) => {
      out
        .write(
          format!(
            "info depth {depth} seldepth {} score {} time {time} nodes {} nps {nps} hashfull {hashfull} pv {}\n",
            settings.seldepth - position.ply_count(),
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
      }))
      .ok();
    }
  }
}

fn alpha_beta_root(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  moves: &Vec<Move>,
  depth: u8,
  out: &mut Output,
) -> (Vec<Move>, Score) {
  let mut alpha = settings.initial_alpha;
  let beta = Score::Win(0);
  let mut best_pv = Vec::new();
  let mut backup_pv = Vec::new();
  let mut move_count = 0;
  let mut show_output = false;
  for candidate in moves {
    let mut position = board.clone();
    position.play_move(*candidate);
    settings.nodes += 1;
    move_count += 1;
    let (mut pv, score) = if move_count > 1 {
      // Zero window search to see if raises alpha
      let score = -zero_window_search(state, settings, &position, depth - 1, -alpha, false);
      if score > alpha {
        if settings.search_is_over() {
          return (best_pv, alpha);
        }
        backup_pv = best_pv;
        best_pv = vec![*candidate];
        if show_output {
          print_info(
            out,
            board,
            alpha,
            depth,
            settings,
            &best_pv,
            state.table.capacity(),
          );
        }
        let (pv, score) = alpha_beta(state, settings, &position, depth - 1, -beta, -alpha, false);
        (pv, -score)
      } else {
        (Vec::new(), score)
      }
    } else {
      let (pv, score) = alpha_beta(
        state,
        settings,
        &position,
        depth - 1,
        Score::Loss(0),
        -alpha,
        false,
      );
      if settings.millis >= 100 {
        show_output = true;
      }
      (pv, -score)
    };
    if settings.search_is_over() {
      return (best_pv, alpha);
    }
    if score > alpha {
      alpha = score;
      let mut new_pv = vec![*candidate];
      new_pv.append(&mut pv);
      best_pv = new_pv;
      backup_pv = best_pv.clone();
      if show_output {
        print_info(
          out,
          board,
          alpha,
          depth,
          settings,
          &best_pv,
          state.table.capacity(),
        );
      }
    } else {
      // In case of PVS research fail-low, revert best pv
      best_pv = backup_pv.clone();
    }
    if !settings.hard_tm && settings.start.elapsed().as_millis() * 5 >= settings.max_time * 4 {
      return (best_pv, alpha);
    }
  }
  (best_pv, alpha)
}

/// Search the specified position and moves to the specified depth
pub fn search(
  state: &mut State,
  mut settings: SearchConfig,
  position: &mut Board,
  mut moves: Vec<Move>,
  mut out: Output,
) -> Vec<Move> {
  position.skip_checkmate = true;
  let mut best_pv = vec![moves[0]];
  let mut current_score = evaluate(state, position);
  let mut depth = 0;
  let mut display_depth = 0;
  while depth < settings.max_depth
    && (settings.hard_tm || settings.start.elapsed().as_millis() * 4 <= settings.max_time)
  {
    depth += 1;
    let (pv, score) = alpha_beta_root(state, &mut settings, position, &moves, depth, &mut out);
    if !pv.is_empty() {
      display_depth = depth;
      best_pv = pv;
      current_score = score;
    } else if !settings.search_is_over() {
      display_depth = depth;
    }
    print_info(
      &mut out,
      position,
      current_score,
      display_depth,
      &settings,
      &best_pv,
      state.table.capacity(),
    );
    if settings.search_is_over() {
      break;
    }
    let bestmove = best_pv[0];
    let mut sorted_moves = vec![bestmove];
    sorted_moves.extend(
      get_move_order(state, position, &moves)
        .into_iter()
        .filter(|m| *m != bestmove),
    );
    moves = sorted_moves;
  }
  best_pv
}

/// Search the specified position to a certain depth and return the node count
pub fn bench(
  state: &mut State,
  board: &mut Board,
  depth: u8,
  qdepth: &mut u8,
  debug: &mut bool,
  rx: &Receiver<Message>,
  mut out: Output,
) -> usize {
  println!("Bench for position {}", board.to_string());
  board.skip_checkmate = true;
  state.new_game(board);
  let mut settings = SearchConfig::new(
    qdepth,
    depth,
    u128::MAX,
    usize::MAX,
    Score::Loss(0),
    true,
    rx,
    debug,
  );
  let (mut moves, mut other) = board.generate_pseudolegal();
  moves.sort_by_key(|(_, piece, capture)| {
    MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
      - 100 * MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
  });
  let mut moves: Vec<Move> = moves.into_iter().map(|(m, _, _)| m).collect();
  moves.append(&mut other);
  let mut best_pv = vec![moves[0]];
  let mut current_score = evaluate(state, board);
  let mut depth = 0;
  while depth < settings.max_depth
    && (settings.start.elapsed().as_millis() * 2 <= settings.max_time)
  {
    depth += 1;
    let (pv, score) = alpha_beta_root(state, &mut settings, board, &moves, depth, &mut out);
    if !pv.is_empty() {
      best_pv = pv;
      current_score = score;
    }
    print_info(
      &mut out,
      board,
      current_score,
      depth,
      &settings,
      &best_pv,
      state.table.capacity(),
    );
    if settings.search_is_over() {
      break;
    }
    let bestmove = best_pv[0];
    let (mut new_moves, mut other) = board.generate_pseudolegal();
    new_moves.sort_by_key(|(_, piece, capture)| {
      MIDDLEGAME_PIECE_VALUES[usize::from(*piece - 1)]
        - 100 * MIDDLEGAME_PIECE_VALUES[usize::from(*capture - 1)]
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
    sorted_moves.extend(new_moves.into_iter().filter(|m| *m != bestmove));
    moves = sorted_moves;
  }
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
        .expect("Board is missing move")
        .to_string()
    );
  }
  println!("Nodes searched: {total}");
  println!("Elapsed time: {}ms", start.elapsed().as_millis());
}
