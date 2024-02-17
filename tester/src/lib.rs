#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A testing program for comparing 2 different engines against each other in a range of positions.

use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};

/// Available options for starting position
pub enum StartingPosition {
  /// Fixed FEN with random moves
  Fen(&'static str),
  /// Randomly generated board
  Random,
}

/// The test positions for the match
pub const POSITIONS: &[(&str, StartingPosition, u32, f64)] = &[
  ("startpos", StartingPosition::Fen(STARTPOS), 18, 0.4996),
  (
    "rectangle",
    StartingPosition::Fen(CAPABLANCA_RECTANGLE),
    18,
    0.374,
  ),
  ("capablanca", StartingPosition::Fen(CAPABLANCA), 24, 0.3426),
  ("liberty", StartingPosition::Fen(LIBERTY_CHESS), 65, 0.1041),
  ("mini", StartingPosition::Fen(MINI), 12, 0.3851),
  ("mongol", StartingPosition::Fen(MONGOL), 24, 0.2498),
  ("african", StartingPosition::Fen(AFRICAN), 24, 0.3546),
  ("narnia", StartingPosition::Fen(NARNIA), 15, 0.4206),
  ("trump", StartingPosition::Fen(TRUMP), 35, 0.2195),
  ("loaded", StartingPosition::Fen(LOADED_BOARD), 12, 0.3094),
  ("double", StartingPosition::Fen(DOUBLE_CHESS), 15, 0.2584),
  ("horde", StartingPosition::Fen(HORDE), 16, 0.3261),
  (
    "elimination",
    StartingPosition::Fen(ELIMINATION),
    25,
    0.02865,
  ),
  (
    "endgame",
    StartingPosition::Fen("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1"),
    12,
    0.7044,
  ),
  ("random", StartingPosition::Random, 24, 0.2874),
];
