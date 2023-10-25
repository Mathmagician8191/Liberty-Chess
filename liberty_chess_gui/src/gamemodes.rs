use crate::helpers::NumericalInput;
use enum_iterator::Sequence;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

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
}

impl Presets {
  pub fn value(self) -> String {
    match self {
      Self::Standard => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      Self::Liberty => "ruabhqkhbcur/wlzenxxnezlw/pppppppppppp/12/12/12/12/12/12/PPPPPPPPPPPP/WLZENXXNEZLW/RUABHQKHBCUR w KQkq - 0 1 3,3 qcaehurwbznxl",
      Self::Mini => "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1 1",
      Self::CapablancaRectangle => "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 - qcarbn",
      Self::CapablancaSquare => "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 3 qcarbn",
      Self::Mongol => "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1 - inzl",
      Self::African => "lnzekznl/pppppppp/8/8/8/8/PPPPPPPP/LNZEKZNL w - - 0 1 - enzl",
      Self::Narnia => "uuqkkquu/pppppppp/8/8/8/8/PPPPPPPP/UUQKKQUU w - - 0 1 - u",
      Self::Trump => "rwwwkwwr/pppppppp/8/8/8/8/PPPPPPPP/RWWWKWWR w KQkq - 0 1 - mrw",
      Self::LoadedBoard => "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1 1",
      Self::Double => "rnbqkbnrrnbqkbnr/pppppppppppppppp/16/16/16/16/PPPPPPPPPPPPPPPP/RNBQKBNRRNBQKBNR w KQkq - 0 1",
      Self::Horde => "rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP w kq - 0 1", 
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
    }
    .to_owned()
  }
}

#[derive(Eq, PartialEq)]
pub struct RandomConfig {
  pub pieces: String,
  pub width: NumericalInput<usize>,
  pub height: NumericalInput<usize>,
}

impl ToString for RandomConfig {
  fn to_string(&self) -> String {
    let width = self.width.get_value();

    // The gap between the white and black pieces
    let gap = self.height.get_value() - 4;

    // The available pieces to choose from
    let pieces = self.pieces.to_lowercase().chars().collect::<Vec<char>>();

    let mut rng = thread_rng();

    // Get the pieces on the board
    let mut pieces: Vec<char> = (0..width)
      .map(|_| *pieces.choose(&mut rng).unwrap_or(&'n'))
      .collect();

    // Add a king to the board
    pieces[rng.gen_range(0..width)] = 'k';

    let pieces = pieces.iter().collect::<String>();

    // Build and return the final L-FEN
    let mut result = pieces.clone();
    result.push('/');
    result += &"p".repeat(width);
    result.push('/');
    result += &(width.to_string() + "/").repeat(gap);
    result += &"P".repeat(width);
    result.push('/');
    result += &pieces.to_uppercase();
    result += " w KQkq - 0 1";
    result
  }
}

impl Default for RandomConfig {
  fn default() -> Self {
    Self {
      pieces: "qrbn".to_owned(),
      width: NumericalInput::<usize>::new(8, 2, usize::MAX),
      height: NumericalInput::<usize>::new(8, 4, usize::MAX),
    }
  }
}
