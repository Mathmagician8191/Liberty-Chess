#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! An engine to handle playing sounds for Liberty Chess

extern crate alloc;

use alloc::string::String;
use kira::manager::backend::cpal::{CpalBackend, Error};
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::tween::Tween;
use kira::LoopBehavior;
use std::io::Cursor;

/// 100%, which is the default volume
pub const DEFAULT_VOLUME: u8 = 100;
const DEFAULT_VOUME_FLOAT: f64 = DEFAULT_VOLUME as f64;

// The paths for music
#[cfg(feature = "music")]
const MUSIC: &[u8] = include_bytes!("../../resources/music/Hydrangeas-for-a-Friend-Calm.ogg");
#[cfg(feature = "music")]
const MUSIC_EXTRA: &[u8] =
  include_bytes!("../../resources/music/Hydrangeas-for-a-Friend-Extra.ogg");

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

fn load_raw(data: &'static [u8], loop_behavior: Option<LoopBehavior>) -> StaticSoundData {
  let settings = StaticSoundSettings::default().loop_behavior(loop_behavior);
  StaticSoundData::from_cursor(Cursor::new(data), settings).unwrap()
}

fn load_audio(data: &'static [u8]) -> StaticSoundData {
  load_raw(data, None)
}

#[cfg(feature = "music")]
fn load_music(data: &'static [u8]) -> StaticSoundData {
  load_raw(
    data,
    Some(LoopBehavior {
      start_position: 0.0,
    }),
  )
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
#[allow(unused_must_use)]
fn set_volume(handle: &mut StaticSoundHandle, volume: f64) {
  handle.set_volume(volume, Tween::default());
}

// If it fails, we don't care
#[allow(unused_must_use)]
#[cfg(feature = "music")]
fn stop(handle: &mut StaticSoundHandle) {
  handle.stop(Tween::default());
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

#[cfg(feature = "music")]
struct MusicTrack {
  handle: StaticSoundHandle,
}

#[cfg(feature = "music")]
impl MusicTrack {
  fn new(player: &mut AudioManager, music: StaticSoundData) -> Option<Self> {
    Some(Self {
      handle: player.play(music).ok()?,
    })
  }
}

#[cfg(feature = "music")]
impl Drop for MusicTrack {
  fn drop(&mut self) {
    stop(&mut self.handle);
  }
}

// takes about 670 ms to load on a 5600x, consider putting on another thread
#[cfg(feature = "music")]
struct MusicPlayer {
  volume: u8,
  dramatic_scale: f64,
  clock_drama: f64,
  calm: MusicTrack,
  extra: Option<MusicTrack>,
}

#[cfg(feature = "music")]
impl MusicPlayer {
  fn new(player: &mut AudioManager, volume: u8, dramatic: bool) -> Option<Self> {
    let (calm, extra) = Self::load_music(player, volume, dramatic)?;
    Some(Self {
      volume,
      dramatic_scale: 0.0,
      clock_drama: 0.0,
      calm,
      extra,
    })
  }

  // Always enables dramatic music
  fn reload(&mut self, player: &mut AudioManager) {
    if let Some((calm, extra)) = Self::load_music(player, self.volume, true) {
      self.calm = calm;
      self.extra = extra;
    }
  }

  fn get_dramatic(&self) -> f64 {
    self.dramatic_scale + self.clock_drama
  }

  fn load_music(
    player: &mut AudioManager,
    volume: u8,
    dramatic: bool,
  ) -> Option<(MusicTrack, Option<MusicTrack>)> {
    let music = load_music(MUSIC);
    let extra = if dramatic {
      let dramatic = load_music(MUSIC_EXTRA);
      let mut extra = MusicTrack::new(player, dramatic)?;
      set_volume(&mut extra.handle, 0.0);
      Some(extra)
    } else {
      None
    };
    let mut calm = MusicTrack::new(player, music)?;
    set_volume(&mut calm.handle, convert_volume(volume));
    Some((calm, extra))
  }

  fn update_dramatic(&mut self) {
    let dramatic = self.get_dramatic();
    if let Some(ref mut track) = self.extra {
      set_volume(&mut track.handle, convert_volume(self.volume) * dramatic);
    }
  }
}

/// The sound engine
pub struct Engine {
  player: AudioManager,
  sound_volume: u8,
  sounds: [StaticSoundData; 10],
  #[cfg(feature = "music")]
  music_player: Option<MusicPlayer>,
}

impl Engine {
  #[must_use]
  fn setup(
    sound_volume: Option<&str>,
    #[cfg(feature = "music")] music_volume: Option<&str>,
    #[cfg(feature = "music")] dramatic: bool,
  ) -> Option<Self> {
    #[cfg(feature = "music")]
    {
      let music_volume = load_volume(music_volume);
      if music_volume != 0 {
        let mut player = get_manager().ok()?;
        let music_player = MusicPlayer::new(&mut player, music_volume, dramatic);
        return Some(Self {
          player,
          sound_volume: load_volume(sound_volume),
          sounds: get_effects(),
          music_player,
        });
      };
    }
    Some(Self {
      player: get_manager().ok()?,
      sound_volume: load_volume(sound_volume),
      sounds: get_effects(),
      #[cfg(feature = "music")]
      music_player: None,
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

  /// Get the current volume for music
  #[cfg(feature = "music")]
  #[must_use]
  pub fn get_music_volume(&self) -> u8 {
    self.music_player.as_ref().map_or(0, |player| player.volume)
  }

  /// Update the current volume for music
  #[cfg(feature = "music")]
  pub fn set_music_volume(&mut self, volume: u8) {
    if let Some(player) = &mut self.music_player {
      player.volume = volume;
      let volume = convert_volume(volume);
      set_volume(&mut player.calm.handle, volume);
      player.update_dramatic();
    }
  }

  /// Update how dramatic the music should be
  #[cfg(feature = "music")]
  pub fn set_dramatic(&mut self, dramatic: f64) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_scale = player.dramatic_scale.mul_add(0.5, dramatic);
      player.update_dramatic();
    }
  }

  /// Reset the drama level of the music to 0
  #[cfg(feature = "music")]
  pub fn clear_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_scale = 0.0;
      player.clock_drama = 0.0;
      player.update_dramatic();
    }
  }

  // Float comparison used to tell if value needs to be updated
  #[allow(clippy::float_cmp)]
  /// Update how dramatic the clock is
  #[cfg(feature = "music")]
  pub fn set_clock_bonus(&mut self, clock_drama: f64) {
    if let Some(player) = &mut self.music_player {
      if clock_drama != player.clock_drama {
        player.clock_drama = clock_drama;
        player.update_dramatic();
      }
    }
  }

  /// Play the specified sound effect
  pub fn play(&mut self, sound: &Effect) {
    let mut handle = self.player.play(
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
    if let Ok(ref mut handle) = handle {
      set_volume(handle, convert_volume(self.sound_volume));
    }
  }

  /// Returns whether music is enabled
  #[cfg(feature = "music")]
  #[must_use]
  pub fn music_enabled(&self) -> bool {
    self.music_player.is_some()
  }

  /// Returns whether dramatic music is enabled
  #[cfg(feature = "music")]
  #[must_use]
  pub fn dramatic_enabled(&self) -> bool {
    self
      .music_player
      .as_ref()
      .map_or(false, |player| player.extra.is_some())
  }

  /// Toggle whether music should play
  #[cfg(feature = "music")]
  pub fn toggle_music(&mut self) {
    self.music_player = match self.music_player {
      Some(_) => None,
      None => MusicPlayer::new(&mut self.player, DEFAULT_VOLUME, true),
    }
  }

  /// Toggle whether dramatic music should play
  #[cfg(feature = "music")]
  pub fn toggle_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      match &player.extra {
        Some(_) => player.extra = None,
        None => player.reload(&mut self.player),
      }
    }
  }
}
