#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A testing program for comparing 2 different engines against each other in a range of positions.

use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};

/// The test positions for the match
pub const POSITIONS: &[(&str, &str, u32, f64)] = &[
  ("startpos", STARTPOS, 18, 0.518),
  ("rectangle", CAPABLANCA_RECTANGLE, 18, 0.3832),
  ("capablanca", CAPABLANCA, 24, 0.3184),
  ("liberty", LIBERTY_CHESS, 65, 0.07995),
  ("mini", MINI, 12, 0.4981),
  ("mongol", MONGOL, 24, 0.2787),
  ("african", AFRICAN, 24, 0.3771),
  ("narnia", NARNIA, 15, 0.4679),
  ("trump", TRUMP, 35, 0.2862),
  ("loaded", LOADED_BOARD, 12, 0.371),
  ("double", DOUBLE_CHESS, 15, 0.2365),
  ("horde", HORDE, 16, 0.2876),
  ("elimination", ELIMINATION, 25, 0.0514),
  (
    "endgame",
    "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",
    12,
    0.7669,
  ),
];
