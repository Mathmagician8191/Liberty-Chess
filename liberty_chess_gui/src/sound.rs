#[cfg(feature = "sound")]
use soloud::{AudioExt, LoadExt, Soloud, Wav};

#[cfg(feature = "sound")]
fn load_audio(data: &[u8]) -> Wav {
  let mut wav = Wav::default();

  wav.load_mem(data).unwrap();

  wav
}

#[cfg(feature = "sound")]
pub fn get() -> [Wav; 3] {
  [
    load_audio(include_bytes!("../../resources/sounds/Move.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Capture.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Check.ogg")),
  ]
}

#[cfg(feature = "sound")]
pub fn get_player(sound: bool) -> Option<Soloud> {
  if sound {
    Soloud::default().ok()
  } else {
    None
  }
}
