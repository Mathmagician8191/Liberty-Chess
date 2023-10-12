#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! An engine to handle playing sounds for Liberty Chess

extern crate alloc;

use alloc::string::String;
use kira::manager::backend::cpal::{CpalBackend, Error};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::Tween;
use std::io::Cursor;

#[cfg(feature = "music")]
use crate::music::Player;
#[cfg(feature = "music")]
use alloc::sync::Arc;
#[cfg(feature = "music")]
use parking_lot::Mutex;

#[cfg(feature = "music")]
mod music;

/// 100%, which is the default volume
pub const DEFAULT_VOLUME: u8 = 100;
const DEFAULT_VOUME_FLOAT: f64 = DEFAULT_VOLUME as f64;

fn convert_volume(volume: u8) -> f64 {
  f64::from(volume) / DEFAULT_VOUME_FLOAT
}

fn load_volume(volume: Option<&str>) -> u8 {
  if let Some(volume) = volume {
    if let Ok(volume) = volume.parse::<u8>() {
      return volume;
    }
  }
  DEFAULT_VOLUME
}

fn load_audio(data: &'static [u8]) -> StaticSoundData {
  let settings = StaticSoundSettings::default();
  StaticSoundData::from_cursor(Cursor::new(data), settings).unwrap()
}

fn get_effects() -> [StaticSoundData; 10] {
  [
    load_audio(include_bytes!("../../resources/sounds/Move.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Illegal.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Capture.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Check.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Victory.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Draw.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Navigate.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Return.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Enable.ogg")),
    load_audio(include_bytes!("../../resources/sounds/Disable.ogg")),
  ]
}

fn get_manager() -> Result<AudioManager, Error> {
  AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
}

// If it fails, we don't care
fn set_volume(handle: &mut StaticSoundHandle, volume: f64) {
  handle.set_volume(volume, Tween::default()).unwrap_or(());
}

/// A sound effect option to play
pub enum Effect {
  /// The default move sound
  Move,
  /// The sound for an illegal move attempt
  Illegal,
  /// The sound for a capture occuring
  Capture,
  /// The sound for check. Has priority over capture if they both occur
  Check,
  /// The sound for victory
  Victory,
  /// The sound for a draw
  Draw,
  /// The sound for navigating to another page
  Navigate,
  /// The sound for returning to the main menu
  Return,
  /// The sound for enabling a checkbox
  Enable,
  /// The sound for disabling a checkbox
  Disable,
}

/// The sound engine
pub struct Engine {
  #[cfg(feature = "music")]
  player: Arc<Mutex<AudioManager>>,
  #[cfg(not(feature = "music"))]
  player: AudioManager,
  sound_volume: u8,
  sounds: [StaticSoundData; 10],
  #[cfg(feature = "music")]
  music_player: Option<Player>,
}

impl Engine {
  #[cfg(feature = "music")]
  #[must_use]
  fn setup(sound_volume: Option<&str>, music_volume: Option<&str>, dramatic: bool) -> Option<Self> {
    let music_volume = load_volume(music_volume);
    let player = Arc::new(Mutex::new(get_manager().ok()?));
    let music_player =
      (music_volume != 0).then(|| Player::new(player.clone(), music_volume, dramatic));
    Some(Self {
      player,
      sound_volume: load_volume(sound_volume),
      sounds: get_effects(),
      music_player,
    })
  }

  #[cfg(not(feature = "music"))]
  #[must_use]
  fn setup(sound_volume: Option<&str>) -> Option<Self> {
    Some(Self {
      player: get_manager().ok()?,
      sound_volume: load_volume(sound_volume),
      sounds: get_effects(),
    })
  }

  /// Initialises the sound engine.
  ///
  /// Returns `None` if it fails to load.
  #[must_use]
  pub fn new() -> Option<Self> {
    Self::setup(
      None,
      #[cfg(feature = "music")]
      None,
      #[cfg(feature = "music")]
      true,
    )
  }

  /// Load the sound engine from existing data.
  ///
  /// Returns `None` if it is disabled or fails to load.
  #[must_use]
  pub fn load(
    enabled: &Option<String>,
    sound_volume: &Option<String>,
    #[cfg(feature = "music")] music_volume: &Option<String>,
    #[cfg(feature = "music")] dramatic: &Option<String>,
  ) -> Option<Self> {
    let enabled = enabled.as_deref() != Some("false");
    #[cfg(feature = "music")]
    let dramatic = dramatic.as_deref() != Some("false");
    if enabled {
      Self::setup(
        sound_volume.as_deref(),
        #[cfg(feature = "music")]
        music_volume.as_deref(),
        #[cfg(feature = "music")]
        dramatic,
      )
    } else {
      None
    }
  }

  /// Get the current volume for sound effects
  #[must_use]
  pub fn get_sound_volume(&self) -> u8 {
    self.sound_volume
  }

  /// Update the current volume for sound effects
  pub fn set_sound_volume(&mut self, volume: u8) {
    self.sound_volume = volume;
  }

  /// Play the specified sound effect
  pub fn play(&mut self, sound: &Effect) {
    #[cfg(feature = "music")]
    let mut player = self.player.lock();
    #[cfg(not(feature = "music"))]
    let player = &mut self.player;
    let mut handle = player.play(
      self.sounds[match *sound {
        Effect::Move => 0,
        Effect::Illegal => 1,
        Effect::Capture => 2,
        Effect::Check => 3,
        Effect::Victory => 4,
        Effect::Draw => 5,
        Effect::Navigate => 6,
        Effect::Return => 7,
        Effect::Enable => 8,
        Effect::Disable => 9,
      }]
      .clone(),
    );
    #[cfg(feature = "music")]
    drop(player);
    if let Ok(ref mut handle) = handle {
      set_volume(handle, convert_volume(self.sound_volume));
    }
  }
}
