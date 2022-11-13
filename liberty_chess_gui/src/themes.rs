use eframe::egui::style::Visuals;
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
    .to_string()
  }
}

impl Theme {
  pub fn get_visuals(self) -> Visuals {
    match self {
      Self::Dark => Visuals::dark(),
      Self::Red => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(255, 0, 0)),
        ..Visuals::dark()
      },
      Self::Yellow => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(255, 255, 0)),
        ..Visuals::dark()
      },
      Self::Green => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(0, 255, 0)),
        ..Visuals::dark()
      },
      Self::Blue => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(0, 0, 255)),
        ..Visuals::dark()
      },
      Self::Purple => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(192, 0, 192)),
        ..Visuals::dark()
      },
      Self::Light => Visuals::light(),
    }
  }
}

pub fn get_theme(theme: Option<String>) -> Theme {
  if let Some(theme) = theme {
    for possible_theme in all::<Theme>() {
      if possible_theme.to_string() == theme {
        return possible_theme;
      }
    }
  }
  Theme::Dark
}
