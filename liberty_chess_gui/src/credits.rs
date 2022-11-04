use enum_iterator::Sequence;

#[derive(Clone, Copy, Sequence)]
pub enum Credits {
  Coding,
  Images,
  #[cfg(feature = "sound")]
  Sound,
}

impl Credits {
  pub fn title(self) -> &'static str {
    match self {
      Credits::Coding => "Coding",
      Credits::Images => "Images",
      #[cfg(feature = "sound")]
      Credits::Sound => "Sound effects",
    }
  }
}
