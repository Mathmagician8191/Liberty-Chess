use eframe::egui::style::Visuals;
use enum_iterator::Sequence;

#[derive(Clone, Copy, PartialEq, Sequence)]
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
      Theme::Dark => "Dark",
      Theme::Red => "Red",
      Theme::Yellow => "Yellow",
      Theme::Green => "Green",
      Theme::Blue => "Blue",
      Theme::Purple => "Purple",
      Theme::Light => "Light",
    }
    .to_string()
  }
}

impl Theme {
  pub fn get_visuals(self) -> Visuals {
    match self {
      Theme::Dark => Visuals::dark(),
      Theme::Red => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(255, 0, 0)),
        ..Visuals::dark()
      },
      Theme::Yellow => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(255, 255, 0)),
        ..Visuals::dark()
      },
      Theme::Green => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(0, 255, 0)),
        ..Visuals::dark()
      },
      Theme::Blue => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(0, 0, 255)),
        ..Visuals::dark()
      },
      Theme::Purple => Visuals {
        override_text_color: Some(eframe::egui::Color32::from_rgb(192, 0, 192)),
        ..Visuals::dark()
      },
      Theme::Light => Visuals::light(),
    }
  }
}
