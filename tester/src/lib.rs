#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A testing program for comparing 2 different engines against each other in a range of positions.

use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};

/// The test positions for the match
pub const POSITIONS: &[(&str, &str, u32, f64)] = &[
  ("startpos", STARTPOS, 20, 0.4395),
  ("rectangle", CAPABLANCA_RECTANGLE, 20, 0.3343),
  ("capablanca", CAPABLANCA, 25, 0.3846),
  ("liberty", LIBERTY_CHESS, 90, 0.09108),
  ("mini", MINI, 12, 0.4289),
  ("mongol", MONGOL, 30, 0.185),
  ("african", AFRICAN, 30, 0.1599),
  ("narnia", NARNIA, 15, 0.4701),
  ("trump", TRUMP, 60, 0.2262),
  ("loaded", LOADED_BOARD, 12, 0.242),
  ("double", DOUBLE_CHESS, 15, 0.2048),
  ("horde", HORDE, 16, 0.2854),
  ("elimination", ELIMINATION, 25, 0.07153),
  (
    "endgame",
    "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",
    12,
    1.0176,
  ),
];
