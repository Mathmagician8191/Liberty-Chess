use enum_iterator::Sequence;

#[derive(Eq, PartialEq)]
pub enum GameMode {
  Preset(Presets),
  Custom,
}

impl ToString for GameMode {
  fn to_string(&self) -> String {
    match self {
      GameMode::Preset(preset) => preset.to_string(),
      GameMode::Custom => "Custom".to_string(),
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
      Presets::Standard => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      Presets::Liberty => "rnabhqkbhcnr/wlzeuxxuezlw/pppppppppppp/12/12/12/12/12/12/PPPPPPPPPPPP/WLZEUXXUEZLW/RNABHQKBHCNR w KQkq - 0 1 3,3",
      Presets::Mini => "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1 1",
      Presets::CapablancaRectangle => "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1",
      Presets::CapablancaSquare => "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 3",
      Presets::Mongol => "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1",
      Presets::African => "lnzekznl/pppppppp/8/8/8/8/PPPPPPPP/LNZEKZNL w - - 0 1 - enzl",
      Presets::Narnia => "uuqkkquu/pppppppp/8/8/8/8/PPPPPPPP/UUQKKQUU w - - 0 1 - u",
      Presets::Trump => "rwwwkwwr/pppppppp/8/8/8/8/PPPPPPPP/RWWWKWWR w KQkq - 0 1 - rw",
      Presets::LoadedBoard => "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1",
      Presets::Double => "rnbqkbnrrnbqkbnr/pppppppppppppppp/16/16/16/16/PPPPPPPPPPPPPPPP/RNBQKBNRRNBQKBNR w KQkq - 0 1",
    }
    .to_string()
  }
}

impl ToString for Presets {
  fn to_string(&self) -> String {
    match self {
      Presets::Standard => "Standard",
      Presets::Liberty => "Liberty chess",
      Presets::Mini => "Mini chess",
      Presets::CapablancaRectangle => "Capablanca's chess (10x8)",
      Presets::CapablancaSquare => "Capablanca's chess (10x10)",
      Presets::Mongol => "Mongol chess",
      Presets::African => "African chess",
      Presets::Narnia => "Narnia chess",
      Presets::Trump => "Trump chess",
      Presets::LoadedBoard => "Loaded board",
      Presets::Double => "Double chess",
    }
    .to_string()
  }
}
