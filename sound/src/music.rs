use crate::{convert_volume, load_audio, set_volume, Engine, DEFAULT_VOLUME};
use alloc::sync::Arc;
use kira::manager::AudioManager;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::tween::Tween;
use parking_lot::Mutex;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

// The paths for music
const MUSIC: [(&[u8], Option<&[u8]>); 6] = [
  (
    include_bytes!("../../resources/music/Hydrangeas-for-a-Friend-Calm.ogg"),
    Some(include_bytes!(
      "../../resources/music/Hydrangeas-for-a-Friend-Extra.ogg"
    )),
  ),
  (
    include_bytes!("../../resources/music/01 - Renard Lullaby.ogg"),
    None,
  ),
  (
    include_bytes!("../../resources/music/02 - Cognitive Ambience.ogg"),
    None,
  ),
  (
    include_bytes!("../../resources/music/03 - Recursive Thinking.ogg"),
    None,
  ),
  (
    include_bytes!("../../resources/music/04 - Perplex Objector.ogg"),
    None,
  ),
  (
    include_bytes!("../../resources/music/05 - Wasted Opportunity.ogg"),
    None,
  ),
];

// If it fails, we don't care
fn stop(handle: &mut StaticSoundHandle) {
  handle.stop(Tween::default()).unwrap_or(());
}

struct MusicTrack {
  handle: StaticSoundHandle,
}

impl MusicTrack {
  fn new(player: &mut AudioManager, music: StaticSoundData) -> Option<Self> {
    Some(Self {
      handle: player.play(music).ok()?,
    })
  }
}

impl Drop for MusicTrack {
  fn drop(&mut self) {
    stop(&mut self.handle);
  }
}

enum MusicMessage {
  Volume(u8),
  Dramatic(f64),
  EnableDramatic,
  DisableDramatic,
  Loop,
  Stop,
}

// takes about 670 ms to load on a 5600x, consider putting on another thread
pub struct Player {
  volume: u8,
  dramatic: bool,
  dramatic_scale: f64,
  clock_drama: f64,
  tx: Sender<MusicMessage>,
}

impl Drop for Player {
  fn drop(&mut self) {
    self.tx.send(MusicMessage::Stop).unwrap_or(());
  }
}

impl Player {
  pub fn new(player: Arc<Mutex<AudioManager>>, volume: u8, dramatic: bool) -> Self {
    let (tx, rx) = channel();
    let new_tx = tx.clone();
    thread::spawn(move || Self::bg_thread(&player, volume, dramatic, &new_tx, &rx));
    Self {
      volume,
      dramatic,
      dramatic_scale: 0.0,
      clock_drama: 0.0,
      tx,
    }
  }

  fn get_dramatic(&self) -> f64 {
    self.dramatic_scale + self.clock_drama
  }

  fn load_music(
    player: &Arc<Mutex<AudioManager>>,
    volume: u8,
    dramatic: bool,
    tx: Sender<MusicMessage>,
  ) -> Option<(MusicTrack, Option<MusicTrack>)> {
    let (music, dramatic_music): (&[u8], Option<&[u8]>) = *MUSIC.choose(&mut thread_rng())?;
    let music = load_audio(music);
    let extra = if dramatic {
      if let Some(dramatic_music) = dramatic_music {
        let dramatic = load_audio(dramatic_music);
        let mut extra = MusicTrack::new(&mut player.lock(), dramatic)?;
        set_volume(&mut extra.handle, 0.0);
        Some(extra)
      } else {
        None
      }
    } else {
      None
    };
    let duration = music.duration();
    thread::spawn(move || {
      thread::sleep(duration);
      tx.send(MusicMessage::Loop).unwrap_or(());
    });
    let mut calm = MusicTrack::new(&mut player.lock(), music)?;
    set_volume(&mut calm.handle, convert_volume(volume));
    Some((calm, extra))
  }

  fn update_dramatic(track: &mut Option<MusicTrack>, volume: f64, dramatic: f64) {
    if let Some(ref mut track) = track {
      set_volume(&mut track.handle, volume * dramatic);
    }
  }

  // The music thread is separate due to the loading time of the music
  fn bg_thread(
    player: &Arc<Mutex<AudioManager>>,
    mut volume: u8,
    mut dramatic: bool,
    tx: &Sender<MusicMessage>,
    rx: &Receiver<MusicMessage>,
  ) -> Option<()> {
    let (mut calm, mut extra) = Self::load_music(player, volume, dramatic, tx.clone())?;
    let mut dramatic_volume = 0.0;
    while let Ok(message) = rx.recv() {
      match message {
        MusicMessage::Volume(new_volume) => {
          volume = new_volume;
          let volume = convert_volume(volume);
          set_volume(&mut calm.handle, volume);
          Self::update_dramatic(&mut extra, volume, dramatic_volume);
        }
        MusicMessage::Dramatic(new_dramatic) => {
          dramatic_volume = new_dramatic;
          Self::update_dramatic(&mut extra, convert_volume(volume), dramatic_volume);
        }
        MusicMessage::DisableDramatic => {
          extra = None;
          dramatic = false;
        }
        MusicMessage::EnableDramatic => {
          dramatic = true;
          (calm, extra) = Self::load_music(player, volume, true, tx.clone())?;
        }
        MusicMessage::Loop => {
          (calm, extra) = Self::load_music(player, volume, dramatic, tx.clone())?;
          if dramatic {
            Self::update_dramatic(&mut extra, convert_volume(volume), dramatic_volume);
          }
        }
        MusicMessage::Stop => return Some(()),
      }
    }
    Some(())
  }
}

impl Engine {
  /// Get the current volume for music
  #[must_use]
  pub fn get_music_volume(&self) -> u8 {
    self.music_player.as_ref().map_or(0, |player| player.volume)
  }

  /// Update the current volume for music
  pub fn set_music_volume(&mut self, volume: u8) {
    if let Some(player) = &mut self.music_player {
      player.volume = volume;
      player.tx.send(MusicMessage::Volume(volume)).unwrap_or(());
    }
  }

  /// Update how dramatic the music should be
  pub fn set_dramatic(&mut self, dramatic: f64) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_scale = player.dramatic_scale.mul_add(0.5, dramatic);
      player
        .tx
        .send(MusicMessage::Dramatic(player.get_dramatic()))
        .unwrap_or(());
    }
  }

  /// Reset the drama level of the music to 0
  pub fn clear_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_scale = 0.0;
      player.clock_drama = 0.0;
      player
        .tx
        .send(MusicMessage::Dramatic(player.get_dramatic()))
        .unwrap_or(());
    }
  }

  // Float comparison used to tell if value needs to be updated
  #[allow(clippy::float_cmp)]
  /// Update how dramatic the clock is
  pub fn set_clock_bonus(&mut self, clock_drama: f64) {
    if let Some(player) = &mut self.music_player {
      if clock_drama != player.clock_drama {
        player.clock_drama = clock_drama;
        player
          .tx
          .send(MusicMessage::Dramatic(player.get_dramatic()))
          .unwrap_or(());
      }
    }
  }

  /// Returns whether music is enabled
  #[must_use]
  pub fn music_enabled(&self) -> bool {
    self.music_player.is_some()
  }

  /// Returns whether dramatic music is enabled
  #[must_use]
  pub fn dramatic_enabled(&self) -> bool {
    self
      .music_player
      .as_ref()
      .map_or(false, |player| player.dramatic)
  }

  /// Toggle whether music should play
  pub fn toggle_music(&mut self) {
    self.music_player = match self.music_player {
      Some(_) => None,
      None => Some(Player::new(self.player.clone(), DEFAULT_VOLUME, true)),
    }
  }

  /// Toggle whether dramatic music should play
  pub fn toggle_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      player
        .tx
        .send(if player.dramatic {
          MusicMessage::DisableDramatic
        } else {
          MusicMessage::EnableDramatic
        })
        .unwrap_or(());
      player.dramatic = !player.dramatic;
    }
  }
}
