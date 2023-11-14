#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! The infrastructure for the ULCI interface, both client and server

use core::ops::Neg;
use liberty_chess::moves::Move;
use liberty_chess::Piece;
use parking_lot::Mutex;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::Write;
use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;

/// The functionality for a ULCI client
pub mod client;
/// The functionality for a ULCI server
pub mod server;

#[cfg(test)]
mod tests;

const VERSION: usize = 1;

/// The information required for the client
pub struct ClientInfo {
  /// The name of the client
  pub name: String,
  /// The username of a human player, `None` if computer
  pub username: Option<String>,
  /// The author of the client
  pub author: String,
  /// Options for the client
  pub options: HashMap<String, UlciOption>,
  /// Pieces supported by the client
  pub pieces: Vec<Piece>,
}

/// Settings for a search
pub struct SearchSettings {
  /// The available moves to search
  pub moves: Vec<Move>,
  /// The time control for searching
  pub time: SearchTime,
}

/// The time control for searching
#[derive(Clone, Copy)]
pub enum SearchTime {
  /// Fixed time per move
  FixedTime(Duration),
  /// Time and increment
  Increment(Duration, Duration),
  /// Depth
  Depth(u8),
  /// Nodes
  Nodes(usize),
  /// Infinite
  Infinite,
}

/// The value of some option to update
pub enum OptionValue {
  // First string name in option is the option name, second parameter is the value
  /// The value of a string option
  UpdateString(String),
  /// The value of an integer option
  UpdateInt(usize),
  /// The value of a true/false option
  UpdateBool(bool),
  /// The value of an option from a range of possibilities
  UpdateRange(String),
  /// A trigger signal for the engine
  SendTrigger,
}

/// An option supported by the client
pub enum UlciOption {
  /// A string option
  String(String),
  /// An integer option
  Int(IntOption),
  /// A true/false option
  Bool(bool),
  /// One of a range of possibilities
  Range(RangeOption),
  /// A trigger button to do something
  Trigger,
}

impl ToString for UlciOption {
  fn to_string(&self) -> String {
    match self {
      Self::String(option) => format!("type string default {option}"),
      Self::Int(option) => option.to_string(),
      Self::Bool(option) => format!("type check default {option}"),
      Self::Range(option) => option.to_string(),
      Self::Trigger => "type button".to_owned(),
    }
  }
}

/// An option with an integer value and optional min/max
pub struct IntOption {
  /// the default value of the option
  pub default: usize,
  /// the minimum value of the option
  pub min: usize,
  /// the maximum value of the option
  pub max: usize,
}

impl ToString for IntOption {
  fn to_string(&self) -> String {
    let mut result = format!("type spin default {}", self.default);
    result += &format!(" min {}", self.min);
    result += &format!(" max {}", self.max);
    result
  }
}

/// One of a range of possibilities
pub struct RangeOption {
  default: String,
  options: HashSet<String>,
}

impl ToString for RangeOption {
  fn to_string(&self) -> String {
    let mut result = format!("type combo default {}", self.default);
    for option in &self.options {
      result += &format!(" var {option}");
    }
    result
  }
}

/// An evaluation of a position
#[derive(Copy, Clone)]
pub enum Score {
  /// Side to move wins in this many moves
  Win(u16),
  /// Side to move loses in this many moves
  Loss(u16),
  /// Side to move has this advantage in centipawns
  Centipawn(f64),
  /// Side to move has these chances to win, draw and loss permill
  WDL(u16, u16, u16),
}

impl PartialEq for Score {
  fn eq(&self, other: &Self) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl PartialOrd for Score {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match self {
      Self::Win(moves) => match other {
        Self::Win(other_moves) => Some(other_moves.cmp(moves)),
        _ => Some(Ordering::Greater),
      },
      Self::Loss(moves) => match other {
        Self::Loss(other_moves) => Some(moves.cmp(other_moves)),
        _ => Some(Ordering::Less),
      },
      Self::Centipawn(score) => match other {
        Self::Win(_) => Some(Ordering::Less),
        Self::Loss(_) => Some(Ordering::Greater),
        Self::Centipawn(other_score) => score.partial_cmp(other_score),
        Self::WDL(_, _, _) => None,
      },
      Self::WDL(win, _, loss) => match other {
        Self::Win(_) => Some(Ordering::Less),
        Self::Loss(_) => Some(Ordering::Greater),
        Self::Centipawn(_) => None,
        Self::WDL(other_win, _, other_loss) => Some((win + other_loss).cmp(&(other_win + loss))),
      },
    }
  }
}

// Designed for use in Negamax, undoes 1 ply
impl Neg for Score {
  type Output = Self;

  fn neg(self) -> Self::Output {
    match self {
      Self::Win(moves) => Self::Loss(moves),
      Self::Loss(moves) => Self::Win(moves + 1),
      Self::Centipawn(score) => Self::Centipawn(-score),
      Self::WDL(w, d, l) => Self::WDL(l, d, w),
    }
  }
}

// Design for decaying alpha/beta
// Reverses the effect of Neg
impl Not for Score {
  type Output = Self;

  fn not(self) -> Self::Output {
    match self {
      Self::Win(moves) => Self::Loss(moves.saturating_sub(1)),
      Self::Loss(moves) => Self::Win(moves),
      Self::Centipawn(score) => Self::Centipawn(-score),
      Self::WDL(w, d, l) => Self::WDL(l, d, w),
    }
  }
}

impl ToString for Score {
  fn to_string(&self) -> String {
    match self {
      Self::Win(moves) => format!("mate {moves}"),
      Self::Loss(moves) => format!("mate -{moves}"),
      Self::Centipawn(cp) => format!("cp {}", cp.round() as i64),
      Self::WDL(w, d, l) => format!("wdl {w} {d} {l}"),
    }
  }
}

fn write(writer: &mut impl Write, output: impl Display) {
  writer.write(format!("{output}\n").as_bytes()).ok();
}

fn write_mutex(writer: &Arc<Mutex<impl Write>>, output: impl Display) {
  writer.lock().write(format!("{output}\n").as_bytes()).ok();
}
