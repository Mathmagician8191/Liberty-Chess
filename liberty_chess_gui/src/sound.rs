use soloud::{AudioExt, LoadExt, Wav};

fn load_audio(data: &[u8]) -> Wav {
  let mut wav = Wav::default();

  wav.load_mem(data).unwrap();

  wav
}

pub fn get() -> [Wav; 2] {
  [
    load_audio(include_bytes!("../../resources/sounds/Move.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Capture.ogg")),
  ]
}
