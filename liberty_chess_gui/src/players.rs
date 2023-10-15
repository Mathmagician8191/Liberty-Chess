use enum_iterator::Sequence;
use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum PlayerType {
  RandomEngine,
}

impl ToString for PlayerType {
  fn to_string(&self) -> String {
    match self {
      Self::RandomEngine => "Random Mover",
    }
    .to_string()
  }
}

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum PlayerColour {
  White,
  Black,
  Random,
}

impl ToString for PlayerColour {
  fn to_string(&self) -> String {
    match self {
      Self::White => "White",
      Self::Black => "Black",
      Self::Random => "Random",
    }
    .to_string()
  }
}

impl PlayerColour {
  pub fn get_colour(&self) -> bool {
    match self {
      Self::White => true,
      Self::Black => false,
      Self::Random => thread_rng().gen(),
    }
  }
}
