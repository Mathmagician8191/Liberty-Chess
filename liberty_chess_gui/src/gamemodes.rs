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
  Mini,
  CapablancaRectangle,
  CapablancaSquare,
  Mongol,
  African,
  LoadedBoard,
}

impl Presets {
  pub fn value(self) -> String {
    match self {
      Presets::Standard => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
      Presets::Mini => "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1 1".to_string(),
      Presets::CapablancaRectangle => {
        "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1".to_string()
      }
      Presets::CapablancaSquare => {
        "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 3".to_string()
      }
      Presets::Mongol => "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1".to_string(),
      Presets::African => "lnzekznl/pppppppp/8/8/8/8/PPPPPPPP/LNZEKZNL w - - 0 1".to_string(),
      Presets::LoadedBoard => {
        "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1"
          .to_string()
      }
    }
  }
}

impl ToString for Presets {
  fn to_string(&self) -> String {
    match self {
      Presets::Standard => "Standard".to_string(),
      Presets::Mini => "Mini chess".to_string(),
      Presets::CapablancaRectangle => "Capablanca's chess (10x8)".to_string(),
      Presets::CapablancaSquare => "Capablanca's chess (10x10)".to_string(),
      Presets::Mongol => "Mongol chess".to_string(),
      Presets::African => "African chess".to_string(),
      Presets::LoadedBoard => "Loaded board".to_string(),
    }
  }
}
