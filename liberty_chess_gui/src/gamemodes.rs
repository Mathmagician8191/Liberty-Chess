use enum_iterator::Sequence;

#[derive(Eq, PartialEq)]
pub enum GameMode {
  Preset(Presets),
  Custom,
}

impl ToString for GameMode {
  fn to_string(&self) -> String {
    match self {
      Self::Preset(preset) => preset.to_string(),
      Self::Custom => "Custom".to_string(),
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
}

impl Presets {
  pub fn value(self) -> String {
    match self {
      Self::Standard => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      Self::Liberty => "ruabhqkhbcur/wlzenxxnezlw/pppppppppppp/12/12/12/12/12/12/PPPPPPPPPPPP/WLZENXXNEZLW/RUABHQKHBCUR w KQkq - 0 1 3,3 qcaehurwbznxl",
      Self::Mini => "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1 1",
      Self::CapablancaRectangle => "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1",
      Self::CapablancaSquare => "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 3",
      Self::Mongol => "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1 - inzl",
      Self::African => "lnzekznl/pppppppp/8/8/8/8/PPPPPPPP/LNZEKZNL w - - 0 1 - enzl",
      Self::Narnia => "uuqkkquu/pppppppp/8/8/8/8/PPPPPPPP/UUQKKQUU w - - 0 1 - u",
      Self::Trump => "rwwwkwwr/pppppppp/8/8/8/8/PPPPPPPP/RWWWKWWR w KQkq - 0 1 - mrw",
      Self::LoadedBoard => "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1",
      Self::Double => "rnbqkbnrrnbqkbnr/pppppppppppppppp/16/16/16/16/PPPPPPPPPPPPPPPP/RNBQKBNRRNBQKBNR w KQkq - 0 1",
    }
    .to_string()
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
    }
    .to_string()
  }
}
