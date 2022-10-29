use eframe::egui::Color32;

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
  pub fn value(&self) -> Color32 {
    match self {
      Colours::BlackSquare => Color32::from_rgb(160, 128, 96),
      Colours::WhiteSquare => Color32::from_rgb(240, 217, 181),
      Colours::Moved => Color32::from_rgb(64, 192, 0),
      Colours::Selected => Color32::from_rgb(192, 192, 0),
      Colours::ValidBlack => Color32::from_rgb(80, 192, 176),
      Colours::ValidWhite => Color32::from_rgb(120, 237, 219),
      Colours::ThreatenedBlack => Color32::from_rgb(180, 64, 0),
      Colours::ThreatenedWhite => Color32::from_rgb(220, 107, 0),
      Colours::Check => Color32::from_rgb(192, 0, 0),
    }
  }
}
