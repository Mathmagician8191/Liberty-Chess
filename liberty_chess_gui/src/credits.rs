use enum_iterator::Sequence;

#[derive(Clone, Copy, Sequence)]
pub enum Credits {
  Coding,
  Images,
}

impl Credits {
  pub fn title(self) -> &'static str {
    match self {
      Credits::Coding => "Coding",
      Credits::Images => "Images",
    }
  }
}
