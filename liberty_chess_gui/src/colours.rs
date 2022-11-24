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
