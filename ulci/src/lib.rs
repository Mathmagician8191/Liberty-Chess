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
use std::sync::Arc;

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
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SearchTime {
  /// Time and increment
  Increment(u128, u128),
  /// Infinite search
  Infinite,
  /// Depth/Nodes/Movetime
  Other(Limits),
}

impl ToString for SearchTime {
  fn to_string(&self) -> String {
    match self {
      Self::Increment(time, inc) => {
        format!("go wtime {time} winc {inc} btime {time} binc {inc}")
      }
      Self::Infinite => "go infinite".to_owned(),
      Self::Other(limits) => {
        let mut result = "go".to_owned();
        let mut limit_count = 0;
        if limits.depth < u8::MAX {
          result += &format!(" depth {}", limits.depth);
          limit_count += 1;
        }
        if limits.nodes < usize::MAX {
          result += &format!(" nodes {}", limits.nodes);
          limit_count += 1;
        }
        if limits.time < u128::MAX {
          result += &format!(" movetime {}", limits.time);
          limit_count += 1;
        }
        if limit_count == 0 {
          result += " infinite";
        }
        result
      }
    }
  }
}

/// Combined depth/modes/movetime limits
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Limits {
  /// Limit search to depth
  pub depth: u8,
  /// Limit search to nodes
  pub nodes: usize,
  /// Limit search to time in ms
  pub time: u128,
}

impl Default for Limits {
  fn default() -> Self {
    Self {
      depth: u8::MAX,
      nodes: usize::MAX,
      time: u128::MAX,
    }
  }
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
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Score {
  /// Side to move wins on this move
  Win(u16),
  /// Side to move loses on this move
  Loss(u16),
  /// Side to move has this advantage in centipawns
  Centipawn(i64),
}

impl PartialOrd for Score {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Score {
  fn cmp(&self, other: &Self) -> Ordering {
    match self {
      Self::Win(moves) => match other {
        Self::Win(other_moves) => other_moves.cmp(moves),
        _ => Ordering::Greater,
      },
      Self::Loss(moves) => match other {
        Self::Loss(other_moves) => moves.cmp(other_moves),
        _ => Ordering::Less,
      },
      Self::Centipawn(score) => match other {
        Self::Win(_) => Ordering::Less,
        Self::Loss(_) => Ordering::Greater,
        Self::Centipawn(other_score) => score.cmp(other_score),
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
      Self::Loss(moves) => Self::Win(moves),
      Self::Centipawn(score) => Self::Centipawn(-score),
    }
  }
}

impl Score {
  /// Uci output for the score
  pub fn show_uci(&self, move_count: u16) -> String {
    match self {
      Self::Win(moves) => format!("mate {}", moves - move_count),
      Self::Loss(moves) => format!("mate -{}", moves - move_count),
      Self::Centipawn(cp) => format!("cp {cp}"),
    }
  }
}

/// Side to move has these chances to win, draw and loss permill
pub struct WDL {
  win: u16,
  draw: u16,
  loss: u16,
}

impl ToString for WDL {
  fn to_string(&self) -> String {
    format!("wdl {} {} {}", self.win, self.draw, self.loss)
  }
}

fn write(writer: &mut impl Write, output: impl Display) {
  writer.write(format!("{output}\n").as_bytes()).ok();
}

fn write_mutex(writer: &Arc<Mutex<impl Write>>, output: impl Display) {
  writer.lock().write(format!("{output}\n").as_bytes()).ok();
}
