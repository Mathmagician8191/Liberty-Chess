use crate::helpers::NumericalInput;
use enum_iterator::Sequence;
use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::random_board::generate;

#[derive(Eq, PartialEq)]
pub enum GameMode {
  Preset(Presets),
  Custom,
  Random(RandomConfig),
}

impl ToString for GameMode {
  fn to_string(&self) -> String {
    match self {
      Self::Preset(preset) => preset.to_string(),
      Self::Custom => "Custom".to_owned(),
      Self::Random(_) => "Random".to_owned(),
    }
  }
}

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum Presets {
  Standard,
  Liberty,
  Mini,
  CapablancaRectangle,
  CapablancaSquare,
  Mongol,
  African,
  Narnia,
  Trump,
  LoadedBoard,
  Double,
  Horde,
  Elimination,
}

impl Presets {
  pub fn value(self) -> String {
    match self {
      Self::Standard => STARTPOS,
      Self::Liberty => LIBERTY_CHESS,
      Self::Mini => MINI,
      Self::CapablancaRectangle => CAPABLANCA_RECTANGLE,
      Self::CapablancaSquare => CAPABLANCA,
      Self::Mongol => MONGOL,
      Self::African => AFRICAN,
      Self::Narnia => NARNIA,
      Self::Trump => TRUMP,
      Self::LoadedBoard => LOADED_BOARD,
      Self::Double => DOUBLE_CHESS,
      Self::Horde => HORDE,
      Self::Elimination => ELIMINATION,
    }
    .to_owned()
  }
}

impl ToString for Presets {
  fn to_string(&self) -> String {
    match self {
      Self::Standard => "Standard",
      Self::Liberty => "Liberty chess",
      Self::Mini => "Mini chess",
      Self::CapablancaRectangle => "Capablanca's chess (10x8)",
      Self::CapablancaSquare => "Capablanca's chess (10x10)",
      Self::Mongol => "Mongol chess",
      Self::African => "African chess",
      Self::Narnia => "Narnia chess",
      Self::Trump => "Trump chess",
      Self::LoadedBoard => "Loaded board",
      Self::Double => "Double chess",
      Self::Horde => "Horde",
      Self::Elimination => "Elimination",
    }
    .to_owned()
  }
}

#[derive(Eq, PartialEq)]
pub struct RandomConfig {
  pub pieces: String,
  pub spawn_king: bool,
  pub width: NumericalInput<usize>,
  pub height: NumericalInput<usize>,
}

impl ToString for RandomConfig {
  fn to_string(&self) -> String {
    let width = self.width.get_value();
    let height = self.height.get_value();
    generate(width, height, &self.pieces, self.spawn_king)
  }
}

impl Default for RandomConfig {
  fn default() -> Self {
    Self {
      pieces: "qrbn".to_owned(),
      spawn_king: true,
      width: NumericalInput::<usize>::new(8, 2, 256),
      height: NumericalInput::<usize>::new(8, 4, 256),
    }
  }
}
