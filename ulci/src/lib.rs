#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! The infrastructure for the ULCI interface, both client and server

use liberty_chess::moves::Move;
use std::time::Duration;

/// The functionality for a ULCI client
pub mod client;

const VERSION: usize = 1;

/// Settings for a search
pub struct SearchSettings {
  /// The available moves to search
  pub moves: Vec<Move>,
  /// The time control for searching
  pub time: SearchTime,
}

/// The time control for searching
pub enum SearchTime {
  /// Fixed time per move
  FixedTime(Duration),
  /// Time and increment
  Increment(Duration, Duration),
  /// Depth
  Depth(u16),
  /// Nodes
  Nodes(usize),
  /// Infinite
  Infinite,
}
