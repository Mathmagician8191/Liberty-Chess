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
      Self::BlackSquare => Color32::from_rgb(160, 128, 96),
      Self::WhiteSquare => Color32::from_rgb(240, 217, 181),
      Self::Moved => Color32::from_rgb(64, 192, 0),
      Self::Selected => Color32::from_rgb(192, 192, 0),
      Self::ValidBlack => Color32::from_rgb(80, 192, 176),
      Self::ValidWhite => Color32::from_rgb(120, 237, 219),
      Self::ThreatenedBlack => Color32::from_rgb(180, 64, 0),
      Self::ThreatenedWhite => Color32::from_rgb(220, 107, 0),
      Self::Check => Color32::from_rgb(192, 0, 0),
    }
  }
}
