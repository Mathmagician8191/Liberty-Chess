#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! The GUI for Liberty Chess

use crate::config::{Configuration, BOARD_KEY};
use crate::credits::Credits;
use crate::gamemodes::{GameMode, Presets, RandomConfig};
use crate::help_page::{HelpPage, draw_help};
use crate::helpers::{
  char_text_edit, checkbox, colour_edit, get_fen, label_text_edit, menu_button,
};
use crate::themes::{Colours, Theme};
use eframe::epaint::{Pos2, TextureId};
use eframe::{egui, App, CreationContext, Frame, Storage};
use egui::{
  Area, Button, CentralPanel, ColorImage, ComboBox, Context, Label, RichText, ScrollArea,
  SidePanel, Slider, TextureHandle, TextureOptions, TopBottomPanel, Ui, Vec2,
};
use enum_iterator::all;
use helpers::{populate_dropdown, populate_dropdown_transform, raw_text_edit};
use liberty_chess::parsing::to_name;
use liberty_chess::{Board, Gamestate, Piece};
use players::{PlayerColour, PlayerData, PlayerType, SearchType};
use crate::render::draw_game;
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{FitTo, Tree};
use themes::CustomTheme;
use ulci::{Limits, SearchTime};

#[cfg(not(feature = "benchmarking"))]
use std::time::Duration;
#[cfg(feature = "benchmarking")]
use std::time::Instant;

#[cfg(feature = "clock")]
use crate::clock::{convert, draw, draw_edit, init_input};
#[cfg(feature = "clock")]
use crate::helpers::NumericalInput;
#[cfg(feature = "clock")]
use liberty_chess::clock::{Clock, Type};

#[cfg(feature = "music")]
use crate::config::{DRAMATIC_ENABLED_KEY, MUSIC_VOLUME_KEY};

#[cfg(feature = "sound")]
use crate::config::{EFFECT_VOLUME_KEY, SOUND_KEY};
#[cfg(feature = "sound")]
use helpers::update_sound;
#[cfg(feature = "sound")]
use sound::{Effect, Engine, DEFAULT_VOLUME};

#[cfg(target_arch = "wasm32")]
use eframe::{WebOptions, WebRunner};

// submodules
mod config;
mod credits;
mod gamemodes;
mod help_page;
mod helpers;
mod images;
mod players;
mod render;
mod themes;

#[cfg(feature = "clock")]
mod clock;

const MAX_TIME: u64 = 360;

#[derive(Eq, PartialEq)]
enum Screen {
  Menu,
  Game(Box<Board>),
  Help,
  Credits,
  Settings,
}

pub(crate) struct LibertyChessGUI {
  // current screen
  screen: Screen,

  // global settings
  config: Configuration,

  // fields for board rendering
  selected: Option<(usize, usize)>,
  drag: Option<((usize, usize), Pos2)>,
  flipped: bool,

  // fields for main menu
  fen: String,
  gamemode: GameMode,
  friendly: bool,
  message: Option<String>,
  #[cfg(feature = "clock")]
  clock_type: Type,
  #[cfg(feature = "clock")]
  clock_data: [NumericalInput<u64>; 4],
  alternate_player: Option<PlayerType>,
  searchsettings: SearchType,
  alternate_player_colour: PlayerColour,

  // fields for game screen
  undo: Vec<Board>,
  #[cfg(feature = "clock")]
  clock: Option<Clock>,
  promotion: Piece,
  player: Option<(PlayerData, bool)>,
  searchtime: SearchTime,

  // fields for other screens
  help_page: HelpPage,
  credits: Credits,

  // images and a render cache - used on game screen
  images: [Tree; 36],
  renders: [Option<TextureHandle>; 36],

  // audio engine
  #[cfg(feature = "sound")]
  audio_engine: Option<Engine>,

  // for measuring FPS
  #[cfg(feature = "benchmarking")]
  instant: Instant,
  #[cfg(feature = "benchmarking")]
  frames: u32,
  #[cfg(feature = "benchmarking")]
  seconds: u64,
}

impl LibertyChessGUI {
  fn new(ctx: &CreationContext) -> Self {
    let config = Configuration::new(ctx);
    let screen = ctx
      .storage
      .and_then(|data| data.get_string(BOARD_KEY))
      .as_ref()
      .and_then(|fen| Board::new(fen).ok())
      .map_or(Screen::Menu, |board| Screen::Game(Box::new(board)));
    #[cfg(feature = "sound")]
    let audio_engine = match ctx.storage {
      Some(data) => Engine::load(
        &data.get_string(SOUND_KEY),
        &data.get_string(EFFECT_VOLUME_KEY),
        #[cfg(feature = "music")]
        &data.get_string(MUSIC_VOLUME_KEY),
        #[cfg(feature = "music")]
        &data.get_string(DRAMATIC_ENABLED_KEY),
      ),
      None => Engine::new(),
    };
    Self {
      screen,

      config,

      selected: None,
      drag: None,
      flipped: false,

      gamemode: GameMode::Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      friendly: false,
      message: None,
      #[cfg(feature = "clock")]
      clock_type: Type::None,
      #[cfg(feature = "clock")]
      clock_data: [(); 4].map(|()| init_input()),
      alternate_player: None,
      searchsettings: SearchType::default(),
      alternate_player_colour: PlayerColour::Random,

      undo: Vec::new(),
      #[cfg(feature = "clock")]
      clock: None,
      promotion: liberty_chess::QUEEN,
      player: None,
      searchtime: SearchTime::Infinite,

      help_page: HelpPage::PawnForward,
      credits: Credits::Coding,

      images: images::get(),
      renders: [(); 36].map(|()| None),

      #[cfg(feature = "sound")]
      audio_engine,

      #[cfg(feature = "benchmarking")]
      instant: Instant::now(),
      #[cfg(feature = "benchmarking")]
      frames: 0,
      #[cfg(feature = "benchmarking")]
      seconds: 0,
    }
  }

  fn get_image(&mut self, ctx: &Context, piece: Piece, size: u32) -> TextureId {
    let index = match piece {
      _ if piece > 0 => (piece - 1) as usize,
      _ => (17 - piece) as usize,
    };
    if let Some(map) = &self.renders[index] {
      if map.size() == [size as usize; 2] {
        return map.id();
      }
    }
    let mut pixmap = Pixmap::new(size, size).expect("SVG is 0x0");
    resvg::render(
      &self.images[index],
      FitTo::Size(size, size),
      Transform::default(),
      pixmap.as_mut(),
    )
    .unwrap();
    let image = ColorImage::from_rgba_unmultiplied([size as usize; 2], pixmap.data());
    let texture = ctx.load_texture("piece", image, TextureOptions::NEAREST);
    let id = texture.id();
    self.renders[index] = Some(texture);
    id
  }
}

impl App for LibertyChessGUI {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    match &self.screen {
      Screen::Game(board) => {
        let board = board.clone();
        SidePanel::right("Sidebar")
          .min_width((f32::from(self.config.get_text_size())).mul_add(4.8, 6.8))
          .resizable(false)
          .show(ctx, |ui| draw_game_sidebar(self, ui, board));
        #[cfg(feature = "clock")]
        if let Some(clock) = &mut self.clock {
          draw(ctx, clock, self.flipped);
        }
      }
      Screen::Help => {
        SidePanel::left("Help menu")
          .resizable(false)
          .show(ctx, |ui| {
            menu_button(self, ui);
            ScrollArea::vertical().show(ui, |ui| {
              for page in all::<HelpPage>() {
                let mut text = RichText::new(page.title());
                if page == self.help_page {
                  text = text.color(Colours::ValidBlack.value());
                }
                if ui.add(Button::new(text).wrap(false)).clicked() {
                  self.help_page = page;
                }
              }
            });
          });
        TopBottomPanel::bottom("Description")
          .resizable(false)
          .show(ctx, |ui| ui.label(self.help_page.description()));
      }
      Screen::Credits => {
        SidePanel::left("Credits menu")
          .resizable(false)
          .show(ctx, |ui| {
            menu_button(self, ui);
            ui.add(Label::new("Credits:").wrap(false));
            for page in all::<Credits>() {
              if ui.add(Button::new(page.title()).wrap(false)).clicked() {
                self.credits = page;
              }
            }
          });
      }
      Screen::Menu | Screen::Settings => (),
    };

    CentralPanel::default().show(ctx, |ui| {
      match &self.screen {
        Screen::Menu => draw_menu(self, ctx, ui),
        Screen::Game(board) => draw_game(self, ctx, *board.clone()),
        Screen::Help => draw_help(self, ctx),
        Screen::Credits => credits::draw(self, ctx, ui),
        Screen::Settings => {
          let width = ui.available_width();
          Area::new("Settings")
            .fixed_pos(((width / 2.0) - 200.0, 0.0))
            .show(ctx, |ui| draw_settings(self, ctx, ui));
        }
      };
    });

    #[cfg(all(feature = "music", any(feature = "clock", target_arch = "wasm32")))]
    if let Some(player) = &mut self.audio_engine {
      #[cfg(feature = "clock")]
      player.set_clock_bonus(get_clock_drama(&mut self.clock));
      #[cfg(target_arch = "wasm32")]
      player.poll();
    }

    // Re-render every 100 ms if clock is ticking or waiting for engine
    #[cfg(not(feature = "benchmarking"))]
    {
      let mut should_poll = if let Some((player, _)) = &self.player {
        match player {
          PlayerData::RandomEngine => false,
          PlayerData::BuiltIn(interface) => interface.is_waiting(),
        }
      } else {
        false
      };
      #[cfg(feature = "clock")]
      if self.clock.is_some() {
        should_poll = true;
      }
      if should_poll {
        ctx.request_repaint_after(Duration::from_millis(100));
      }
    }

    // Add no delay between rendering frames and log FPS when benchmarking
    #[cfg(feature = "benchmarking")]
    {
      self.frames += 1;
      let duration = self.instant.elapsed().as_secs();
      if duration - self.seconds > 0 {
        self.seconds = duration;
        println!("{} FPS", self.frames);
        self.frames = 0;
      }
      ctx.request_repaint();
    }
  }

  fn save(&mut self, storage: &mut dyn Storage) {
    self.config.save(storage);
    storage.set_string(BOARD_KEY, get_fen(self));
    #[cfg(feature = "sound")]
    {
      storage.set_string(SOUND_KEY, self.audio_engine.is_some().to_string());
      if let Some(engine) = &self.audio_engine {
        storage.set_string(EFFECT_VOLUME_KEY, engine.get_sound_volume().to_string());
        #[cfg(feature = "music")]
        {
          storage.set_string(DRAMATIC_ENABLED_KEY, engine.dramatic_enabled().to_string());
          storage.set_string(MUSIC_VOLUME_KEY, engine.get_music_volume().to_string());
        }
      }
    }
  }
}

fn switch_screen(gui: &mut LibertyChessGUI, screen: Screen) {
  match &gui.screen {
    Screen::Menu => gui.message = None,
    Screen::Game(_) => {
      gui.selected = None;
      gui.undo = Vec::new();
      gui.player = None;
      #[cfg(feature = "clock")]
      {
        gui.clock = None;
      }
      #[cfg(feature = "music")]
      if let Some(ref mut player) = gui.audio_engine {
        player.clear_dramatic();
      }
    }
    Screen::Help => gui.selected = None,
    Screen::Credits | Screen::Settings => (),
  }
  #[cfg(feature = "sound")]
  if let Some(player) = &mut gui.audio_engine {
    player.play(&if screen == Screen::Menu {
      Effect::Return
    } else {
      Effect::Navigate
    });
  }
  gui.screen = screen;
}

fn draw_nav_buttons(gui: &mut LibertyChessGUI, ui: &mut Ui) {
  ui.horizontal_top(|ui| {
    if ui.button("Help").clicked() {
      switch_screen(gui, Screen::Help);
    }
    if ui.button("Credits").clicked() {
      switch_screen(gui, Screen::Credits);
    }
    if ui.button("Settings").clicked() {
      switch_screen(gui, Screen::Settings);
    }
  });
}

// draw main areas for each screen
fn draw_menu(gui: &mut LibertyChessGUI, _ctx: &Context, ui: &mut Ui) {
  draw_nav_buttons(gui, ui);
  ComboBox::from_id_source("Gamemode")
    .selected_text("Gamemode: ".to_owned() + &gui.gamemode.to_string())
    .show_ui(ui, |ui| {
      populate_dropdown_transform(ui, &mut gui.gamemode, GameMode::Preset);
      ui.selectable_value(&mut gui.gamemode, GameMode::Custom, "Custom");
      ui.selectable_value(
        &mut gui.gamemode,
        GameMode::Random(RandomConfig::default()),
        "Random",
      );
    });
  let size = f32::from(gui.config.get_text_size());
  match gui.gamemode {
    GameMode::Preset(ref preset) => {
      gui.fen = preset.value();
    }
    GameMode::Custom => {
      char_text_edit(ui, size, &mut gui.fen);
    }
    GameMode::Random(ref mut config) => {
      char_text_edit(ui, size, &mut config.pieces);
      let size = size * 1.5;
      label_text_edit(ui, size, &mut config.width, "Width");
      label_text_edit(ui, size, &mut config.height, "Height");
    }
  }
  checkbox(
    ui,
    &mut gui.friendly,
    "Friendly Fire",
    #[cfg(feature = "sound")]
    gui.audio_engine.as_mut(),
  );
  if ui.button("Start Game").clicked() {
    if let GameMode::Random(ref config) = gui.gamemode {
      gui.fen = config.to_string();
    }
    match Board::new(&gui.fen) {
      Ok(mut board) => {
        #[cfg(feature = "clock")]
        match gui.clock_type {
          Type::None => gui.clock = None,
          Type::Increment | Type::Handicap => {
            gui.clock = Some(Clock::new(convert(&gui.clock_data), board.to_move()));
          }
        }
        if gui.friendly {
          board.friendly_fire = true;
        }
        #[cfg(feature = "music")]
        if let Some(ref mut player) = gui.audio_engine {
          player.set_dramatic(get_dramatic(&board));
        }

        if gui.config.get_autoflip() {
          gui.flipped = !board.to_move();
        }

        let (player, message) = gui
          .alternate_player
          .as_ref()
          .map_or((None, None), |player| {
            let colour = gui.alternate_player_colour.get_colour();
            if gui.config.get_opponentflip() {
              gui.flipped = colour;
            }
            #[cfg(not(feature = "clock"))]
            let searchtime = gui.searchsettings.get_value();
            #[cfg(feature = "clock")]
            let (searchtime, clock) = gui.searchsettings.get_value(colour);
            #[cfg(feature = "clock")]
            if let Some(clock) = clock {
              let mut clock = Clock::new(clock, board.to_move());
              if !board.to_move() ^ colour {
                clock.toggle_pause();
              }
              gui.clock = Some(clock);
            }
            if searchtime == SearchTime::Other(Limits::default()) {
              (None, Some("Must limit depth, nodes or time".to_owned()))
            } else {
              gui.searchtime = searchtime;
              (Some((PlayerData::new(player), colour)), None)
            }
          });

        gui.player = player;

        if message.is_none() {
          switch_screen(gui, Screen::Game(Box::new(board)));
        }
        gui.message = message;
      }
      Err(error) => {
        gui.message = Some(error.to_string());
      }
    }
  }
  if let Some(message) = &gui.message {
    ui.label(message);
  }

  #[cfg(feature = "clock")]
  if let Some(PlayerType::BuiltIn(_, _)) = gui.alternate_player {
    gui.clock_type = Type::None;
  } else {
    draw_edit(gui, ui, size * 2.0);
  }

  let player_name = gui
    .alternate_player
    .as_ref()
    .map_or_else(|| "Local Opponent".to_string(), ToString::to_string);

  ComboBox::from_id_source("Opponent")
    .selected_text(format!("Opponent: {player_name}"))
    .show_ui(ui, |ui| {
      ui.selectable_value(&mut gui.alternate_player, None, "Local Opponent");
      let values = [PlayerType::RandomEngine, PlayerType::built_in()];
      for value in values {
        let string = value.to_string();
        ui.selectable_value(&mut gui.alternate_player, Some(value), string);
      }
    });

  if gui.alternate_player.is_some() {
    ComboBox::from_id_source("Opponent Colour")
      .selected_text(format!(
        "Colour: {}",
        gui.alternate_player_colour.to_string()
      ))
      .show_ui(ui, |ui| {
        populate_dropdown(ui, &mut gui.alternate_player_colour);
      });
  }
  if let Some(PlayerType::BuiltIn(ref mut qdepth, ref mut hash_size)) = gui.alternate_player {
    ComboBox::from_id_source("Searchtime")
      .selected_text(format!("Searchtime: {}", gui.searchsettings.to_string()))
      .show_ui(ui, |ui| {
        let values = [
          SearchType::default(),
          #[cfg(feature = "clock")]
          SearchType::increment(1, 2),
          #[cfg(feature = "clock")]
          SearchType::handicap(10, 10, 1, 2),
        ];
        for value in values {
          let string = value.to_string();
          ui.selectable_value(&mut gui.searchsettings, value, string);
        }
      });
    match gui.searchsettings {
      #[cfg(feature = "clock")]
      SearchType::Increment(ref mut time, ref mut inc) => {
        ui.horizontal_top(|ui| {
          ui.label("Initial time (minutes)");
          raw_text_edit(ui, size * 3.0, time);
        });
        ui.horizontal_top(|ui| {
          ui.label("Increment (seconds)");
          raw_text_edit(ui, size * 3.0, inc);
        });
      }
      #[cfg(feature = "clock")]
      SearchType::Handicap(
        ref mut human_time,
        ref mut human_inc,
        ref mut engine_time,
        ref mut engine_inc,
      ) => {
        ui.horizontal_top(|ui| {
          ui.label("Human time (minutes)");
          raw_text_edit(ui, size * 3.0, human_time);
          ui.label("Human increment (seconds)");
          raw_text_edit(ui, size * 3.0, human_inc);
        });
        ui.horizontal_top(|ui| {
          ui.label("Engine time (minutes)");
          raw_text_edit(ui, size * 3.0, engine_time);
          ui.label("Engine increment (seconds)");
          raw_text_edit(ui, size * 3.0, engine_inc);
        });
      }
      SearchType::Other(ref mut limits) => {
        ui.horizontal_top(|ui| {
          if checkbox(
            ui,
            &mut limits.depth.is_some(),
            "Limit search by depth",
            #[cfg(feature = "sound")]
            gui.audio_engine.as_mut(),
          ) {
            if limits.depth.is_some() {
              limits.depth = None;
            } else {
              limits.depth = Some(SearchType::depth());
            }
          }
          if let Some(ref mut depth) = limits.depth {
            raw_text_edit(ui, size * 2.0, depth);
          }
        });
        ui.horizontal_top(|ui| {
          if checkbox(
            ui,
            &mut limits.nodes.is_some(),
            "Limit search by nodes",
            #[cfg(feature = "sound")]
            gui.audio_engine.as_mut(),
          ) {
            if limits.nodes.is_some() {
              limits.nodes = None;
            } else {
              limits.nodes = Some(SearchType::nodes());
            }
          }
          if let Some(ref mut nodes) = limits.nodes {
            raw_text_edit(ui, size * 5.0, nodes);
          }
        });
        ui.horizontal_top(|ui| {
          if checkbox(
            ui,
            &mut limits.time.is_some(),
            "Limit search by time (ms)",
            #[cfg(feature = "sound")]
            gui.audio_engine.as_mut(),
          ) {
            if limits.time.is_some() {
              limits.time = None;
            } else {
              limits.time = Some(SearchType::time());
            }
          }
          if let Some(ref mut time) = limits.time {
            raw_text_edit(ui, size * 3.0, time);
          }
        });
      }
    }
    if gui.config.get_advanced() {
      ui.horizontal_top(|ui| {
        ui.label("Quiescence depth");
        raw_text_edit(ui, size * 2.0, qdepth);
      });
      ui.horizontal_top(|ui| {
        ui.label("Hash size (MB)");
        raw_text_edit(ui, size * 4.0, hash_size);
      });
    }
  }
}

fn draw_settings(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui) {
  let mut new_theme = gui.config.get_theme();
  menu_button(gui, ui);
  ComboBox::from_id_source("Theme")
    .selected_text("Theme: ".to_owned() + &new_theme.show())
    .show_ui(ui, |ui| {
      populate_dropdown_transform(ui, &mut new_theme, Theme::Preset);
      ui.selectable_value(
        &mut new_theme,
        Theme::Custom(CustomTheme::new(gui.config.get_theme())),
        "Custom",
      );
    });
  match new_theme {
    Theme::Preset(_) => (),
    Theme::Custom(ref mut custom) => {
      colour_edit(ui, &mut custom.background, "Background");
      colour_edit(ui, &mut custom.text, "Text");
    }
  }
  if gui.config.get_theme() != new_theme {
    gui.config.set_theme(ctx, new_theme);
  }
  let mut size = gui.config.get_text_size();
  if ui
    .add(Slider::new(&mut size, 16..=36).text("Font size"))
    .changed()
  {
    gui.config.set_text_size(ctx, size);
  }
  if checkbox(
    ui,
    &mut gui.config.get_numbers(),
    "Show rank/file numbers",
    #[cfg(feature = "sound")]
    gui.audio_engine.as_mut(),
  ) {
    gui.config.toggle_numbers();
  }
  if checkbox(
    ui,
    &mut gui.config.get_autoflip(),
    "Flip board to side to move",
    #[cfg(feature = "sound")]
    gui.audio_engine.as_mut(),
  ) {
    gui.config.toggle_autoflip();
  }
  if checkbox(
    ui,
    &mut gui.config.get_opponentflip(),
    "Flip board to local player side",
    #[cfg(feature = "sound")]
    gui.audio_engine.as_mut(),
  ) {
    gui.config.toggle_opponentflip();
  }
  #[cfg(feature = "sound")]
  {
    let mut sound = gui.audio_engine.is_some();
    if checkbox(ui, &mut sound, "Sound", None) {
      gui.audio_engine = if sound { Engine::new() } else { None }
    };
    if let Some(ref mut engine) = gui.audio_engine {
      let mut volume = engine.get_sound_volume();
      if ui
        .add(Slider::new(&mut volume, 0..=DEFAULT_VOLUME).text("Move Volume"))
        .changed()
      {
        engine.set_sound_volume(volume);
      }
      #[cfg(feature = "music")]
      {
        let mut music = engine.music_enabled();
        if checkbox(ui, &mut music, "Music", Some(engine)) {
          engine.toggle_music();
        }
        if music {
          if checkbox(
            ui,
            &mut engine.dramatic_enabled(),
            "Dramatic Music",
            Some(engine),
          ) {
            engine.toggle_dramatic();
          }
          let mut volume = engine.get_music_volume();
          if ui
            .add(Slider::new(&mut volume, 0..=DEFAULT_VOLUME).text("Music Volume"))
            .changed()
          {
            engine.set_music_volume(volume);
          }
        }
      }
    }
  }
  if checkbox(
    ui,
    &mut gui.config.get_advanced(),
    "Show advanced engine settings",
    #[cfg(feature = "sound")]
    gui.audio_engine.as_mut(),
  ) {
    gui.config.toggle_advanced();
  }
  //Currently non-functional due to https://github.com/emilk/egui/issues/2641
  //if gui.config.settings_changed() && ui.button("Reset all").clicked() {
  //  gui.config.reset_all(ctx);
  //}
}

// draw areas for specific screens

fn draw_game_sidebar(gui: &mut LibertyChessGUI, ui: &mut Ui, mut gamestate: Box<Board>) {
  menu_button(gui, ui);
  if ui.button("Flip board").clicked() {
    gui.flipped = !gui.flipped;
  }
  if !gui.undo.is_empty() && ui.button("Undo").clicked() {
    let gamestate = gui.undo.pop().expect("Scrodinger's vector");
    #[cfg(feature = "music")]
    if let Some(ref mut player) = gui.audio_engine {
      player.set_dramatic(get_dramatic(&gamestate));
    }
    gui.screen = Screen::Game(Box::new(gamestate));
    if let Some((player, _)) = &mut gui.player {
      match player {
        PlayerData::RandomEngine => (),
        PlayerData::BuiltIn(interface) => interface.cancel_move(),
      }
    }
    #[cfg(feature = "clock")]
    if let Some(clock) = &mut gui.clock {
      if gui.player.is_none() {
        clock.switch_clocks();
      } else if clock.is_paused() {
        clock.toggle_pause();
      }
    };
  }

  #[cfg(feature = "clock")]
  if let Some(clock) = &mut gui.clock {
    if gamestate.state() == Gamestate::InProgress && !clock.is_flagged() {
      let text = if clock.is_paused() {
        "Unpause"
      } else {
        "Pause"
      };
      if ui.button(text).clicked() {
        clock.toggle_pause();
      }
    }
  }

  // display promotion if applicable
  if gamestate.promotion_available() {
    let promotion = gamestate.promotion_options();
    if !promotion.contains(&gui.promotion) {
      gui.promotion = promotion[0];
    }
    ComboBox::from_id_source("Promote")
      .selected_text(to_name(gui.promotion))
      .show_ui(ui, |ui| {
        for piece in promotion {
          ui.selectable_value(&mut gui.promotion, *piece, to_name(*piece));
        }
      });
    if ui.button("Promote").clicked() {
      gamestate.promote(gui.promotion);
      gui.screen = Screen::Game(gamestate.clone());
      #[cfg(feature = "sound")]
      if let Some(engine) = &mut gui.audio_engine {
        let mut effect = Effect::Illegal;
        update_sound(&mut effect, &gamestate, false);
        engine.play(&effect);
      }
      #[cfg(feature = "clock")]
      if let Some(clock) = &mut gui.clock {
        clock.update_status(&gamestate);
      }
    }
  }

  // let the user copy the FEN to clipboard
  #[cfg(not(target_arch = "wasm32"))]
  if ui.button("Copy FEN").clicked() {
    ui.output_mut(|o| o.copied_text = get_fen(gui));
  }

  // if the game is over, report the reason
  let state = gamestate.state();
  ui.label(match state {
    Gamestate::Checkmate(winner) => {
      if winner {
        "White wins by checkmate"
      } else {
        "Black wins by checkmate"
      }
    }
    Gamestate::Stalemate => "Draw by stalemate",
    Gamestate::Move50 => "Draw by 50 move rule",
    Gamestate::Repetition => "Draw by 3-fold repetition",
    Gamestate::Elimination(winner) => {
      if winner {
        "White wins by elimination"
      } else {
        "Black wins by elimination"
      }
    }
    Gamestate::Material => "Draw by insufficient material",
    Gamestate::InProgress => {
      if gamestate.to_move() {
        "White to move"
      } else {
        "Black to move"
      }
    }
  });
}

// general helper functions

#[cfg(feature = "music")]
fn get_dramatic(board: &Board) -> f64 {
  let mut dramatic = 0.0;
  if board.state() != Gamestate::InProgress {
    dramatic += 0.5;
  }
  if board.in_check() {
    dramatic += 0.5;
  }
  dramatic
}

#[cfg(all(feature = "clock", feature = "music"))]
fn get_clock_drama(clock: &mut Option<Clock>) -> f64 {
  clock.as_mut().map_or(0.0, |clock| {
    let data = clock.get_clocks();
    let data = if clock.to_move() { data.0 } else { data.1 };
    // Running out of time is dramatic
    // Returns a linear scale from 0 at 30s to 0.75 at 0s
    if clock.is_paused() {
      0.0
    } else {
      u128::saturating_sub(30000, data.as_millis()) as f64 / 40000.0
    }
  })
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
  let web_options = WebOptions::default();

  wasm_bindgen_futures::spawn_local(async {
    WebRunner::new()
      .start(
        "Liberty Chess", // hardcode it
        web_options,
        Box::new(|cc| Box::new(LibertyChessGUI::new(cc))),
      )
      .await
      .expect("failed to start eframe");
  });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
  let size = helpers::ICON_SIZE;
  let mut pixmap = Pixmap::new(size, size).unwrap();
  resvg::render(
    &images::get()[11],
    FitTo::Size(size, size),
    Transform::default(),
    pixmap.as_mut(),
  )
  .unwrap();
  let options = eframe::NativeOptions {
    // disable vsync for uncapped framerates while benchmarking
    vsync: cfg!(not(feature = "benchmarking")),
    icon_data: Some(eframe::IconData {
      rgba: Pixmap::take(pixmap),
      width: size,
      height: size,
    }),
    min_window_size: Some(Vec2::new(640.0, 480.0)),
    ..Default::default()
  };
  eframe::run_native(
    "Liberty Chess",
    options,
    Box::new(|cc| Box::new(LibertyChessGUI::new(cc))),
  )
  .expect("Failed to load Liberty Chess");
}
