#[cfg(feature = "sound")]
use soloud::{AudioExt, LoadExt, Wav};

#[cfg(feature = "sound")]
fn load_audio(data: &[u8]) -> Wav {
  let mut wav = Wav::default();

  wav.load_mem(data).unwrap();

  wav
}

#[cfg(feature = "sound")]
pub fn get() -> [Wav; 2] {
  [
    load_audio(include_bytes!("../../resources/sounds/Move.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Capture.ogg")),
  ]
}
