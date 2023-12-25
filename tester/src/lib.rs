#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A testing program for comparing 2 different engines against each other in a range of positions.

use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};

/// The test positions for the match
pub const POSITIONS: &[(&str, &str, u32, f64)] = &[
  ("startpos", STARTPOS, 20, 0.4531),
  ("rectangle", CAPABLANCA_RECTANGLE, 20, 0.3114),
  ("capablanca", CAPABLANCA, 25, 0.2909),
  ("liberty", LIBERTY_CHESS, 90, 0.08094),
  ("mini", MINI, 12, 0.4783),
  ("mongol", MONGOL, 30, 0.2336),
  ("african", AFRICAN, 30, 0.4329),
  ("narnia", NARNIA, 15, 0.415),
  ("trump", TRUMP, 60, 0.2642),
  ("loaded", LOADED_BOARD, 12, 0.3222),
  ("double", DOUBLE_CHESS, 15, 0.2113),
  ("horde", HORDE, 16, 0.2372),
  ("elimination", ELIMINATION, 25, 0.07523),
  (
    "endgame",
    "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",
    12,
    0.7554,
  ),
];
