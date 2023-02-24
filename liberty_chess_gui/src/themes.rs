use core::str::FromStr;
use eframe::egui::{style::Visuals, Color32};
use enum_iterator::{all, Sequence};

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum Theme {
  Dark,
  Red,
  Yellow,
  Green,
  Blue,
  Purple,
  Light,
}

pub enum ThemeError {
  NotFound,
}

impl ToString for Theme {
  fn to_string(&self) -> String {
    match self {
      Self::Dark => "Dark",
      Self::Red => "Red",
      Self::Yellow => "Yellow",
      Self::Green => "Green",
      Self::Blue => "Blue",
      Self::Purple => "Purple",
      Self::Light => "Light",
    }
    .to_owned()
  }
}

impl Theme {
  pub fn get_visuals(self) -> Visuals {
    match self {
      Self::Dark => Visuals::dark(),
      Self::Red => Visuals {
        override_text_color: Some(Color32::from_rgb(255, 0, 0)),
        ..Visuals::dark()
      },
      Self::Yellow => Visuals {
        override_text_color: Some(Color32::from_rgb(255, 255, 0)),
        ..Visuals::dark()
      },
      Self::Green => Visuals {
        override_text_color: Some(Color32::from_rgb(0, 255, 0)),
        ..Visuals::dark()
      },
      Self::Blue => Visuals {
        override_text_color: Some(Color32::from_rgb(0, 0, 255)),
        ..Visuals::dark()
      },
      Self::Purple => Visuals {
        override_text_color: Some(Color32::from_rgb(192, 0, 192)),
        ..Visuals::dark()
      },
      Self::Light => Visuals::light(),
    }
  }
}

impl FromStr for Theme {
  type Err = ThemeError;

  fn from_str(theme: &str) -> Result<Self, ThemeError> {
    all::<Self>()
      .find(|&possible_theme| possible_theme.to_string() == theme)
      .ok_or(ThemeError::NotFound)
  }
}

#[derive(PartialEq, Eq)]
pub enum Colours {
  BlackSquare,
  WhiteSquare,
  Moved,
  Selected,
  ValidBlack,
  ValidWhite,
  ThreatenedBlack,
  ThreatenedWhite,
  Check,
}

impl Colours {
  pub const fn value(&self) -> Color32 {
    match self {
      Self::BlackSquare => Color32::from_rgb(200, 148, 96),
      Self::WhiteSquare => Color32::from_rgb(240, 217, 181),
      Self::Moved => Color32::from_rgb(64, 192, 0),
      Self::Selected => Color32::from_rgb(192, 192, 0),
      Self::ValidBlack => Color32::from_rgb(100, 182, 176),
      Self::ValidWhite => Color32::from_rgb(120, 237, 219),
      Self::ThreatenedBlack => Color32::from_rgb(180, 74, 0),
      Self::ThreatenedWhite => Color32::from_rgb(200, 107, 0),
      Self::Check => Color32::from_rgb(192, 0, 0),
    }
  }
}
