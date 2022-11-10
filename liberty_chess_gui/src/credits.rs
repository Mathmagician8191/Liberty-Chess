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
      Self::Coding => "Coding",
      Self::Images => "Images",
      #[cfg(feature = "sound")]
      Self::Sound => "Sound effects",
    }
  }
}
