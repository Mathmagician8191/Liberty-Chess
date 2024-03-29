#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A testing program for comparing 2 different engines against each other in a range of positions.

use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::random_board::generate;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::Board;
use oxidation::evaluate::evaluate;
use oxidation::parameters::DEFAULT_PARAMETERS;
use oxidation::search::quiescence;
use oxidation::{random_move, SearchConfig, State, QDEPTH};
use rand::{thread_rng, Rng};
use std::num::NonZeroUsize;
use std::sync::mpsc::channel;
use std::thread::available_parallelism;
use threadpool::ThreadPool;
use ulci::{Score, SearchTime};

const RANDOM_MOVE_COUNT: usize = 6;
const FILTER_THRESHOLD: i32 = 200;

/// 1+0.01 for speedups
pub const VSTC: SearchTime = SearchTime::Increment(1000, 10);
/// 8+0.08 for most tests
pub const STC: SearchTime = SearchTime::Increment(8000, 80);
/// 40+0.4 for progression tests and checking scaling
pub const LTC: SearchTime = SearchTime::Increment(40000, 400);

/// The test positions for the match
pub const POSITIONS: &[(&str, StartingPosition, u32)] = &[
  ("startpos", StartingPosition::Fen(STARTPOS), 18),
  ("rectangle", StartingPosition::Fen(CAPABLANCA_RECTANGLE), 18),
  ("capablanca", StartingPosition::Fen(CAPABLANCA), 24),
  ("liberty", StartingPosition::Fen(LIBERTY_CHESS), 65),
  ("mini", StartingPosition::Fen(MINI), 12),
  ("mongol", StartingPosition::Fen(MONGOL), 24),
  ("african", StartingPosition::Fen(AFRICAN), 24),
  ("narnia", StartingPosition::Fen(NARNIA), 15),
  ("trump", StartingPosition::Fen(TRUMP), 35),
  ("loaded", StartingPosition::Fen(LOADED_BOARD), 12),
  ("double", StartingPosition::Fen(DOUBLE_CHESS), 15),
  ("horde", StartingPosition::Fen(HORDE), 16),
  ("elimination", StartingPosition::Fen(ELIMINATION), 25),
  (
    "endgame",
    StartingPosition::Fen("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1"),
    12,
  ),
  ("random", StartingPosition::Random, 24),
];

/// The result of a game in a match
pub enum GameResult {
  /// Player 1 wins
  ChampWin,
  /// Draw
  Draw,
  /// Player 2 wins
  ChallengeWin,
}

/// Available options for starting position
pub enum StartingPosition {
  /// Fixed FEN with random moves
  Fen(&'static str),
  /// Randomly generated board
  Random,
}

impl StartingPosition {
  /// Convert a starting position to an actual board
  #[must_use]
  pub fn get_position(&self, friendly_fire: bool) -> CompressedBoard {
    match self {
      Self::Fen(fen) => {
        let mut board = Board::new(fen).expect("Loading board failed");
        board.friendly_fire = friendly_fire;
        let state = State::new(0, &board, DEFAULT_PARAMETERS);
        let mut debug = false;
        let mut qdepth = QDEPTH;
        let (_tx, rx_2) = channel();
        let mut settings =
          SearchConfig::new_time(&board, &mut qdepth, SearchTime::Infinite, &rx_2, &mut debug);
        let mut eval = evaluate(&state, &board);
        if RANDOM_MOVE_COUNT % 2 == 1 {
          // Final board is opposite stm, invert score
          eval = -eval;
        }
        let alpha = match eval {
          Score::Centipawn(value) => Score::Centipawn(value - FILTER_THRESHOLD),
          _ => eval,
        };
        let beta = match eval {
          Score::Centipawn(value) => Score::Centipawn(value + FILTER_THRESHOLD),
          _ => eval,
        };
        let board = loop {
          let mut board = board.clone();
          for _ in 0..RANDOM_MOVE_COUNT {
            if let Some(randommove) = random_move(&board) {
              if let Some(new_board) = board.move_if_legal(randommove) {
                board = new_board;
              }
            }
          }
          // Filter out busted openings
          let (_, score) = quiescence(&state, &mut settings, &board, QDEPTH, alpha, beta);
          if score > alpha && score < beta {
            break board;
          }
        };
        board.send_to_thread()
      }
      Self::Random => {
        let mut rng = thread_rng();
        let width = rng.gen_range(6..=12);
        let height = rng.gen_range(6..=12);
        let fen = generate(width, height, "mqcaehuriwbznxlo", true);
        let mut board = Board::new(&fen).expect("Loading board failed");
        board.friendly_fire = friendly_fire;
        board.send_to_thread()
      }
    }
  }
}

/// Get a threadpool to execute tasks with
#[must_use]
pub fn get_threadpool() -> ThreadPool {
  let cores = available_parallelism().map_or(1, NonZeroUsize::get);
  ThreadPool::new(cores - 1)
}
