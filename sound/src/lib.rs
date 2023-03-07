#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![no_std]
//! An engine to handle playing sounds for Liberty Chess

extern crate alloc;

use alloc::string::String;
use soloud::{AudioExt, LoadExt, Soloud, Wav};

#[cfg(feature = "music")]
use soloud::Handle;

/// 100%, which is the default volume
pub const DEFAULT_VOLUME: u8 = 100;
const DEFAULT_VOUME_FLOAT: f32 = DEFAULT_VOLUME as f32;

// The paths for music
#[cfg(feature = "music")]
const MUSIC: &[u8] = include_bytes!("../../resources/music/Hydrangeas-for-a-Friend-Calm.ogg");
#[cfg(feature = "music")]
const MUSIC_EXTRA: &[u8] =
  include_bytes!("../../resources/music/Hydrangeas-for-a-Friend-Extra.ogg");

fn convert_volume(volume: u8) -> f32 {
  f32::from(volume) / DEFAULT_VOUME_FLOAT
}

fn load_volume(volume: Option<&str>) -> u8 {
  if let Some(volume) = volume {
    if let Ok(volume) = volume.parse::<u8>() {
      return volume;
    }
  }
  DEFAULT_VOLUME
}

fn load_audio(data: &[u8]) -> Wav {
  let mut wav = Wav::default();
  wav.load_mem(data).unwrap();
  wav
}

fn get_effects() -> [Wav; 10] {
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
  music: Wav,
  handle: Handle,
}

#[cfg(feature = "music")]
impl MusicTrack {
  fn new(player: &Soloud, music: Wav) -> Self {
    let handle = player.play(&music);
    Self { music, handle }
  }

  fn reload(&mut self, player: &mut Soloud, volume: f32) {
    let handle = player.play(&self.music);
    player.set_volume(handle, volume);
    self.handle = handle;
  }
}

// takes about 750 ms to load, check again when replacing with lewton
#[cfg(feature = "music")]
struct MusicPlayer {
  volume: u8,
  dramatic_scale: f32,
  clock_drama: f32,
  calm: MusicTrack,
  extra: Option<MusicTrack>,
}

#[cfg(feature = "music")]
impl MusicPlayer {
  fn new(player: &mut Soloud, volume: u8, dramatic: bool) -> Self {
    let music = load_audio(MUSIC);
    let extra = if dramatic {
      Some(Self::load_dramatic(player))
    } else {
      None
    };
    let calm = MusicTrack::new(player, music);
    player.set_volume(calm.handle, convert_volume(volume));
    Self {
      volume,
      dramatic_scale: 0.0,
      clock_drama: 0.0,
      calm,
      extra,
    }
  }

  fn refresh_music(&mut self, player: &mut Soloud) {
    let volume = convert_volume(self.volume);
    player.stop(self.calm.handle);
    self.calm.reload(player, volume);
    let drama = self.get_dramatic();
    if let Some(extra) = &mut self.extra {
      player.stop(extra.handle);
      extra.reload(player, volume * drama);
    }
  }

  fn get_dramatic(&self) -> f32 {
    self.dramatic_scale + self.clock_drama
  }

  fn load_dramatic(player: &mut Soloud) -> MusicTrack {
    let dramatic = load_audio(MUSIC_EXTRA);
    let extra = MusicTrack::new(player, dramatic);
    player.set_volume(extra.handle, 0.0);
    extra
  }

  fn update_dramatic(&self, player: &mut Soloud) {
    if let Some(track) = &self.extra {
      player.set_volume(
        track.handle,
        convert_volume(self.volume) * self.get_dramatic(),
      );
    }
  }
}

/// The sound engine
pub struct Engine {
  player: Soloud,
  sound_volume: u8,
  sounds: [Wav; 10],
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
        let mut player = Soloud::default().ok()?;
        let music_player = MusicPlayer::new(&mut player, music_volume, dramatic);
        return Some(Self {
          player,
          sound_volume: load_volume(sound_volume),
          sounds: get_effects(),
          music_player: Some(music_player),
        });
      };
    }
    Some(Self {
      player: Soloud::default().ok()?,
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
      self.player.set_volume(player.calm.handle, volume);
      player.update_dramatic(&mut self.player);
    }
  }

  /// Update how dramatic the music should be
  #[cfg(feature = "music")]
  pub fn set_dramatic(&mut self, dramatic: f32) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_scale = player.dramatic_scale.mul_add(0.5, dramatic);
      player.update_dramatic(&mut self.player);
    }
  }

  /// Reset the drama level of the music to 0
  #[cfg(feature = "music")]
  pub fn clear_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_scale = 0.0;
      player.clock_drama = 0.0;
      player.update_dramatic(&mut self.player);
    }
  }

  // Float comparison used to tell if value needs to be updated
  #[allow(clippy::float_cmp)]
  /// Update how dramatic the clock is
  #[cfg(feature = "music")]
  pub fn set_clock_bonus(&mut self, clock_drama: f32) {
    if let Some(player) = &mut self.music_player {
      if clock_drama != player.clock_drama {
        player.clock_drama = clock_drama;
        player.update_dramatic(&mut self.player);
      }
    }
  }

  /// Play the specified sound effect
  pub fn play(&mut self, sound: &Effect) {
    let handle = self.player.play(
      &self.sounds[match *sound {
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
      }],
    );
    self
      .player
      .set_volume(handle, convert_volume(self.sound_volume));
  }

  /// Refresh the music if it has stopped playing. Call regularly to make sure the music loops.
  #[cfg(feature = "music")]
  pub fn update_music(&mut self) {
    if let Some(player) = &mut self.music_player {
      if self.player.active_voice_count() == 0 {
        player.refresh_music(&mut self.player);
      }
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
      None => Some(MusicPlayer::new(&mut self.player, DEFAULT_VOLUME, true)),
    }
  }

  /// Toggle whether dramatic music should play
  #[cfg(feature = "music")]
  pub fn toggle_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      match &player.extra {
        Some(_) => player.extra = None,
        None => {
          player.extra = Some(MusicPlayer::load_dramatic(&mut self.player));
          player.refresh_music(&mut self.player);
        }
      }
    }
  }
}
