use crate::{convert_volume, load_audio, set_volume, Engine, DEFAULT_VOLUME};
use kira::manager::AudioManager;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::tween::Tween;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[cfg(not(feature = "multithreading"))]
use std::cell::RefCell;
#[cfg(not(feature = "multithreading"))]
use std::rc::Rc;

#[cfg(feature = "multithreading")]
use alloc::sync::Arc;
#[cfg(feature = "multithreading")]
use parking_lot::Mutex;
#[cfg(feature = "multithreading")]
use std::sync::mpsc::{channel, Receiver, Sender};
#[cfg(feature = "multithreading")]
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

#[cfg(feature = "multithreading")]
enum MusicMessage {
  Volume(u8),
  Dramatic(f64),
  EnableDramatic,
  DisableDramatic,
  Loop(u32),
  Stop,
}

// loading occurs on another thread due to the time it takes
pub struct Player {
  volume: u8,
  dramatic: bool,
  dramatic_volume: f64,
  clock_drama: f64,
  #[cfg(feature = "multithreading")]
  tx: Sender<MusicMessage>,
  #[cfg(not(feature = "multithreading"))]
  player: Rc<RefCell<AudioManager>>,
  #[cfg(not(feature = "multithreading"))]
  calm: MusicTrack,
  #[cfg(not(feature = "multithreading"))]
  extra: Option<MusicTrack>,
}

#[cfg(feature = "multithreading")]
impl Drop for Player {
  fn drop(&mut self) {
    self.tx.send(MusicMessage::Stop).unwrap_or(());
  }
}

impl Player {
  fn get_dramatic(&self) -> f64 {
    self.dramatic_volume + self.clock_drama
  }

  fn update_dramatic(track: &mut Option<MusicTrack>, volume: f64, dramatic: f64) {
    if let Some(ref mut track) = track {
      set_volume(&mut track.handle, volume * dramatic);
    }
  }

  fn change_volume(
    volume: u8,
    dramatic_volume: f64,
    calm: &mut MusicTrack,
    extra: &mut Option<MusicTrack>,
  ) {
    let volume = convert_volume(volume);
    set_volume(&mut calm.handle, volume);
    Self::update_dramatic(extra, volume, dramatic_volume);
  }
}

#[cfg(feature = "multithreading")]
impl Player {
  pub fn new(player: Arc<Mutex<AudioManager>>, volume: u8, dramatic: bool) -> Option<Self> {
    let (tx, rx) = channel();
    let new_tx = tx.clone();
    thread::spawn(move || Self::bg_thread(&player, volume, dramatic, &new_tx, &rx));
    Some(Self {
      volume,
      dramatic,
      dramatic_volume: 0.0,
      clock_drama: 0.0,
      tx,
    })
  }

  fn load_music(
    player: &Arc<Mutex<AudioManager>>,
    volume: u8,
    dramatic: bool,
    tx: Sender<MusicMessage>,
    counter: &mut u32,
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
    *counter += 1;
    let counter_copy = *counter;
    thread::spawn(move || {
      thread::sleep(duration);
      tx.send(MusicMessage::Loop(counter_copy)).unwrap_or(());
    });
    let mut calm = MusicTrack::new(&mut player.lock(), music)?;
    set_volume(&mut calm.handle, convert_volume(volume));
    Some((calm, extra))
  }

  // The music thread is separate due to the loading time of the music
  fn bg_thread(
    player: &Arc<Mutex<AudioManager>>,
    mut volume: u8,
    mut dramatic: bool,
    tx: &Sender<MusicMessage>,
    rx: &Receiver<MusicMessage>,
  ) -> Option<()> {
    let mut counter = 0;
    let (mut calm, mut extra) =
      Self::load_music(player, volume, dramatic, tx.clone(), &mut counter)?;
    let mut dramatic_volume = 0.0;
    while let Ok(message) = rx.recv() {
      match message {
        MusicMessage::Volume(new_volume) => {
          volume = new_volume;
          Self::change_volume(new_volume, dramatic_volume, &mut calm, &mut extra);
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
          (calm, extra) = Self::load_music(player, volume, true, tx.clone(), &mut counter)?;
        }
        MusicMessage::Loop(index) => {
          if index == counter {
            (calm, extra) = Self::load_music(player, volume, dramatic, tx.clone(), &mut counter)?;
            if dramatic {
              Self::update_dramatic(&mut extra, convert_volume(volume), dramatic_volume);
            }
          }
        }
        MusicMessage::Stop => return Some(()),
      }
    }
    Some(())
  }
}

#[cfg(not(feature = "multithreading"))]
impl Player {
  pub fn new(player: Rc<RefCell<AudioManager>>, volume: u8, dramatic: bool) -> Option<Self> {
    let (calm, extra) = Self::load_music(&player, volume, dramatic)?;
    Some(Self {
      dramatic,
      dramatic_volume: 0.0,
      clock_drama: 0.0,
      player,
      calm,
      extra,
      volume,
    })
  }

  fn load_music(
    player: &Rc<RefCell<AudioManager>>,
    volume: u8,
    dramatic: bool,
  ) -> Option<(MusicTrack, Option<MusicTrack>)> {
    let (music, dramatic_music): (&[u8], Option<&[u8]>) = *MUSIC.choose(&mut thread_rng())?;
    let music = load_audio(music);
    let extra = if dramatic {
      if let Some(dramatic_music) = dramatic_music {
        let dramatic = load_audio(dramatic_music);
        let mut extra = MusicTrack::new(&mut player.borrow_mut(), dramatic)?;
        set_volume(&mut extra.handle, 0.0);
        Some(extra)
      } else {
        None
      }
    } else {
      None
    };
    let mut calm = MusicTrack::new(&mut player.borrow_mut(), music)?;
    set_volume(&mut calm.handle, convert_volume(volume));
    Some((calm, extra))
  }
}

impl Engine {
  /// Get the current volume for music
  #[must_use]
  pub fn get_music_volume(&self) -> u8 {
    self.music_player.as_ref().map_or(0, |player| player.volume)
  }

  /// Returns whether music is enabled
  #[must_use]
  pub const fn music_enabled(&self) -> bool {
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
      None => Player::new(self.player.clone(), DEFAULT_VOLUME, true),
    }
  }
}

#[cfg(feature = "multithreading")]
impl Engine {
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
      player.dramatic_volume = player.dramatic_volume.mul_add(0.5, dramatic);
      player
        .tx
        .send(MusicMessage::Dramatic(player.get_dramatic()))
        .unwrap_or(());
    }
  }

  /// Reset the drama level of the music to 0
  pub fn clear_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_volume = 0.0;
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

#[cfg(not(feature = "multithreading"))]
impl Engine {
  /// Update the current volume for music
  pub fn set_music_volume(&mut self, volume: u8) {
    if let Some(player) = &mut self.music_player {
      player.volume = volume;
      Player::change_volume(
        player.volume,
        player.dramatic_volume,
        &mut player.calm,
        &mut player.extra,
      )
    }
  }

  /// Update how dramatic the music should be
  pub fn set_dramatic(&mut self, dramatic: f64) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_volume = player.dramatic_volume.mul_add(0.5, dramatic);
      let dramatic = player.get_dramatic();
      Player::update_dramatic(&mut player.extra, convert_volume(player.volume), dramatic);
    }
  }

  /// Reset the drama level of the music to 0
  pub fn clear_dramatic(&mut self) {
    if let Some(player) = &mut self.music_player {
      player.dramatic_volume = 0.0;
      player.clock_drama = 0.0;
      let dramatic = player.get_dramatic();
      Player::update_dramatic(&mut player.extra, convert_volume(player.volume), dramatic);
    }
  }

  // Float comparison used to tell if value needs to be updated
  #[allow(clippy::float_cmp)]
  /// Update how dramatic the clock is
  pub fn set_clock_bonus(&mut self, clock_drama: f64) {
    if let Some(player) = &mut self.music_player {
      if clock_drama != player.clock_drama {
        player.clock_drama = clock_drama;
        let dramatic = player.get_dramatic();
        Player::update_dramatic(&mut player.extra, convert_volume(player.volume), dramatic);
      }
    }
  }

  /// Toggle whether dramatic music should play
  pub fn toggle_dramatic(&mut self) -> Option<()> {
    if let Some(player) = &mut self.music_player {
      if player.dramatic {
        player.extra = None;
      } else {
        (player.calm, player.extra) = Player::load_music(&player.player, player.volume, true)?;
      };
      player.dramatic = !player.dramatic;
    }
    Some(())
  }

  /// Poll to see if the music needs to be refreshed
  pub fn poll(&mut self) -> Option<()> {
    if let Some(music_player) = &mut self.music_player {
      if self.player.borrow().num_sounds() == 0 {
        (music_player.calm, music_player.extra) =
          Player::load_music(&self.player, music_player.volume, music_player.dramatic)?;
        if music_player.dramatic {
          Player::update_dramatic(
            &mut music_player.extra,
            convert_volume(music_player.volume),
            music_player.dramatic_volume,
          );
        }
      }
    }
    Some(())
  }
}
