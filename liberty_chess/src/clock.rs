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

  last_update: Instant,
}

impl Clock {
  /// Initialise a `Clock`.
  /// Time Values are in seconds.
  #[must_use]
  pub fn new([white_clock, black_clock, white_inc, black_inc]: [u64; 4], to_move: bool) -> Self {
    Self {
      white_clock: Duration::from_secs(60 * white_clock),
      black_clock: Duration::from_secs(60 * black_clock),
      white_inc: Duration::from_secs(white_inc),
      black_inc: Duration::from_secs(black_inc),
      to_move,
      flagged: false,
      last_update: Instant::now(),
    }
  }

  /// Updates the internal state of the clock.
  pub fn update(&mut self) {
    let elapsed = self.last_update.elapsed();
    self.last_update = Instant::now();
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

  /// Returns whether the clock has flagged.
  /// For accurate results, ensure the clock is updated first.
  #[must_use]
  pub fn is_flagged(&self) -> bool {
    self.flagged
  }

  /// Return the side to move according to the clock.
  #[must_use]
  pub fn to_move(&self) -> bool {
    self.to_move
  }

  /// Updates the clock and returns each player's current time.
  #[must_use]
  pub fn get_clocks(&mut self) -> (Duration, Duration) {
    self.update();
    (self.white_clock, self.black_clock)
  }

  /// Update the clock and switch the clock that is running.
  pub fn switch_clocks(&mut self) {
    self.update();
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
}

/// A type of clock to use
#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum Type {
  None,
  Increment,
  Handicap,
}

impl ToString for Type {
  fn to_string(&self) -> String {
    match self {
      Type::None => "No Clock".to_string(),
      Type::Increment => "Increment".to_string(),
      Type::Handicap => "Increment with Handicap".to_string(),
    }
  }
}

/// Convert a number of seconds to a MM:SS time.
#[must_use]
pub fn print_secs(secs: u64) -> String {
  (secs / 60).to_string() + &format!(":{:0>2}", secs % 60)
}
