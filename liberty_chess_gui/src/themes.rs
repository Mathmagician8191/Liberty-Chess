use core::str::FromStr;
use eframe::egui;
use egui::style::Visuals;
use egui::Color32;
use enum_iterator::{all, Sequence};

fn rgba_to_string(rgba: Color32) -> String {
  let colour = rgba.to_array();
  let mut result = 0;
  for (i, value) in colour.iter().enumerate() {
    result += u32::from(*value) << (24 - 8 * i);
  }
  format!("{result:X}")
}

pub trait GetVisuals {
  fn get_visuals(&self) -> Visuals;
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Theme {
  Preset(PresetTheme),
  Custom(CustomTheme),
}

impl Theme {
  pub fn show(&self) -> String {
    match self {
      Self::Preset(preset) => preset.to_string(),
      Self::Custom(_) => "Custom".to_owned(),
    }
  }
}

impl GetVisuals for Theme {
  fn get_visuals(&self) -> Visuals {
    match self {
      Self::Preset(preset) => preset.get_visuals(),
      Self::Custom(custom) => custom.get_visuals(),
    }
  }
}

impl ToString for Theme {
  fn to_string(&self) -> String {
    match self {
      Self::Preset(preset) => preset.to_string(),
      Self::Custom(custom) => custom.to_string(),
    }
  }
}

impl FromStr for Theme {
  type Err = ();

  fn from_str(theme: &str) -> Result<Self, Self::Err> {
    theme.parse::<PresetTheme>().map_or_else(
      |()| Ok(Self::Custom(theme.parse::<CustomTheme>()?)),
      |theme| Ok(Self::Preset(theme)),
    )
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CustomTheme {
  pub background: Color32,
  pub text: Color32,
}

impl CustomTheme {
  pub fn new(theme: Theme) -> Self {
    match theme {
      Theme::Preset(preset) => preset.get_custom(),
      Theme::Custom(custom) => custom,
    }
  }
}

impl GetVisuals for CustomTheme {
  fn get_visuals(&self) -> Visuals {
    Visuals {
      override_text_color: Some(self.text),
      panel_fill: self.background,
      ..Visuals::dark()
    }
  }
}

impl ToString for CustomTheme {
  fn to_string(&self) -> String {
    rgba_to_string(self.background) + &rgba_to_string(self.text)
  }
}

impl FromStr for CustomTheme {
  type Err = ();

  #[allow(clippy::cast_possible_truncation)]
  fn from_str(theme: &str) -> Result<Self, Self::Err> {
    let value = u64::from_str_radix(theme, 16).map_err(|_| ())?;
    Ok(Self {
      background: Color32::from_rgba_unmultiplied(
        (value >> 56) as u8,
        (value >> 48) as u8,
        (value >> 40) as u8,
        (value >> 32) as u8,
      ),
      text: Color32::from_rgba_unmultiplied(
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
      ),
    })
  }
}

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum PresetTheme {
  Dark,
  Red,
  Yellow,
  Green,
  Blue,
  Purple,
  Light,
}

impl PresetTheme {
  fn get_custom(self) -> CustomTheme {
    let visuals = self.get_visuals();
    CustomTheme {
      background: visuals.panel_fill,
      text: visuals.text_color(),
    }
  }
}

impl ToString for PresetTheme {
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

impl GetVisuals for PresetTheme {
  fn get_visuals(&self) -> Visuals {
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

impl FromStr for PresetTheme {
  type Err = ();

  fn from_str(theme: &str) -> Result<Self, Self::Err> {
    all::<Self>()
      .find(|&possible_theme| possible_theme.to_string() == theme)
      .ok_or(())
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
