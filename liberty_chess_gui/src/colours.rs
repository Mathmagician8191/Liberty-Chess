use eframe::egui::Color32;

pub enum Colours {
  BlackSquare,
  WhiteSquare,
  Moved,
  Selected,
  ValidMoveBlack,
  ValidMoveWhite,
  Threatened,
  Check,
}

impl Colours {
  pub fn value(&self) -> Color32 {
    match self {
      Colours::BlackSquare => Color32::from_rgb(160, 128, 96),
      Colours::WhiteSquare => Color32::from_rgb(240, 217, 181),
      Colours::Moved => Color32::from_rgb(64, 192, 0),
      Colours::Selected => Color32::from_rgb(192, 192, 0),
      Colours::ValidMoveBlack => Color32::from_rgb(120, 144, 108),
      Colours::ValidMoveWhite => Color32::from_rgb(180, 225, 168),
      Colours::Threatened => Color32::from_rgb(200, 128, 0),
      Colours::Check => Color32::from_rgb(192, 0, 0),
    }
  }
}
