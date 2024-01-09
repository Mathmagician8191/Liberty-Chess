use crate::{Board, Gamestate};
use enum_iterator::Sequence;
use std::time::{Duration, Instant};

/// Implements a chess clock.
pub struct Clock {
  white_clock: Duration,
  black_clock: Duration,

  white_inc: Duration,
  black_inc: Duration,

  to_move: bool,
  flagged: bool,
  paused: bool,

  last_update: Instant,
}

impl Clock {
  /// Initialise a `Clock`.
  #[must_use]
  pub fn new(
    [white_clock, black_clock, white_inc, black_inc]: [Duration; 4],
    to_move: bool,
  ) -> Self {
    Self {
      white_clock,
      black_clock,
      white_inc,
      black_inc,
      to_move,
      flagged: false,
      paused: true,
      last_update: Instant::now(),
    }
  }

  /// Initialise a `Clock` where the white and black time is the same
  #[must_use]
  pub fn new_symmetric(clock: Duration, increment: Duration, to_move: bool) -> Self {
    Self::new([clock, clock, increment, increment], to_move)
  }

  /// Updates the internal state of the clock.
  pub fn update(&mut self) {
    let elapsed = self.last_update.elapsed();
    self.last_update = Instant::now();
    if !self.paused {
      if self.to_move {
        if elapsed > self.white_clock {
          self.white_clock = Duration::ZERO;
          self.flagged = true;
        } else {
          self.white_clock -= elapsed;
        }
      } else if elapsed > self.black_clock {
        self.black_clock = Duration::ZERO;
        self.flagged = true;
      } else {
        self.black_clock -= elapsed;
      }
    }
  }

  /// Updates the clock and toggles whether it is paused.
  pub fn toggle_pause(&mut self) {
    self.update();
    self.paused = !self.paused;
  }

  /// Returns whether the clock has flagged.
  /// For accurate results, ensure the clock is updated first.
  #[must_use]
  pub const fn is_flagged(&self) -> bool {
    self.flagged
  }

  /// Returns whether the clock is paused.
  #[must_use]
  pub const fn is_paused(&self) -> bool {
    self.paused
  }

  /// Return the side to move according to the clock.
  #[must_use]
  pub const fn to_move(&self) -> bool {
    self.to_move
  }

  /// Updates the clock and returns each player's current time.
  #[must_use]
  pub fn get_clocks(&mut self) -> (Duration, Duration) {
    self.update();
    (self.white_clock, self.black_clock)
  }

  /// Returns each player's increment
  #[must_use]
  pub const fn get_increment(&self) -> (Duration, Duration) {
    (self.white_inc, self.black_inc)
  }

  /// Update the clock and switch the clock that is running.
  pub fn switch_clocks(&mut self) {
    self.update();
    self.paused = false;
    if !self.flagged {
      if self.to_move {
        self.white_clock += self.white_inc;
        self.to_move = false;
      } else {
        self.black_clock += self.black_inc;
        self.to_move = true;
      }
    }
  }

  /// Update the clock status when a move occurs
  pub fn update_status(&mut self, board: &Board) {
    self.switch_clocks();
    if board.state() != Gamestate::InProgress && !self.is_paused() {
      self.toggle_pause();
    }
  }
}

/// A type of clock to use
#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum Type {
  /// No clock
  None,
  /// Basic Fischer increment
  Increment,
  /// Fischer increment where both sides have differing amounts of time and increment.
  Handicap,
}

impl ToString for Type {
  fn to_string(&self) -> String {
    match self {
      Self::None => "None".to_owned(),
      Self::Increment => "Increment".to_owned(),
      Self::Handicap => "Handicap".to_owned(),
    }
  }
}

/// Convert time in milliseconds to a user-friendly output
#[must_use]
pub fn format_time(millis: u128) -> String {
  let secs = millis / 1000;
  if secs >= 180 {
    format!("{secs} s")
  } else if secs >= 20 {
    format!("{secs}.{} s", (millis / 100) % 10)
  } else {
    format!("{millis} ms")
  }
}
