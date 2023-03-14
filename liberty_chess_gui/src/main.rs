#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! The GUI for Liberty Chess

use crate::config::{Configuration, BOARD_KEY};
use crate::credits::Credits;
use crate::gamemodes::{GameMode, Presets, RandomConfig};
use crate::help_page::HelpPage;
use crate::helpers::{
  char_text_edit, checkbox, colour_edit, get_fen, label_text_edit, menu_button, UV,
};
use crate::themes::{Colours, PresetTheme, Theme};
use eframe::epaint::{Pos2, Shape};
use eframe::{egui, App, CreationContext, Frame, Storage};
use egui::{
  pos2, Align2, Area, Button, CentralPanel, Color32, ColorImage, ComboBox, Context, Label,
  PointerButton, Rect, Response, RichText, Rounding, ScrollArea, Sense, SidePanel, Slider,
  TextureHandle, TextureId, TextureOptions, TopBottomPanel, Ui, Vec2,
};
use enum_iterator::all;
use liberty_chess::{to_name, Board, Gamestate, Piece};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{FitTo, Tree};
use themes::CustomTheme;

#[cfg(feature = "benchmarking")]
use std::time::Instant;

#[cfg(feature = "clock")]
use crate::clock::{draw_clock, draw_clock_edit};
#[cfg(feature = "clock")]
use liberty_chess::clock::{Clock, Type};

#[cfg(feature = "music")]
use crate::config::{DRAMATIC_ENABLED_KEY, MUSIC_VOLUME_KEY};

#[cfg(feature = "sound")]
use crate::config::{EFFECT_VOLUME_KEY, SOUND_KEY};
#[cfg(feature = "sound")]
use sound::{Effect, Engine, DEFAULT_VOLUME};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

// submodules
mod config;
mod credits;
mod gamemodes;
mod help_page;
mod helpers;
mod images;
mod themes;

#[cfg(feature = "clock")]
mod clock;

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
  moved: Option<[(usize, usize); 2]>,
  flipped: bool,

  // fields for main menu
  fen: String,
  gamemode: GameMode,
  friendly: bool,
  message: Option<String>,
  #[cfg(feature = "clock")]
  clock_type: Type,
  #[cfg(feature = "clock")]
  clock_data: [u64; 4],

  // fields for game screen
  undo: Vec<Board>,
  #[cfg(feature = "clock")]
  clock: Option<Clock>,
  promotion: Piece,

  // fields for different screens
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
      moved: None,
      flipped: false,

      gamemode: GameMode::Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      friendly: false,
      message: None,
      #[cfg(feature = "clock")]
      clock_type: Type::None,
      #[cfg(feature = "clock")]
      clock_data: [10; 4],

      undo: Vec::new(),
      #[cfg(feature = "clock")]
      clock: None,
      promotion: liberty_chess::QUEEN,

      help_page: HelpPage::PawnForward,
      credits: Credits::Coding,

      images: images::get(),
      renders: [(); 36].map(|_| None),

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
          .min_width((f32::from(self.config.get_text_size())).mul_add(4.45, 6.8))
          .resizable(false)
          .show(ctx, |ui| draw_game_sidebar(self, ui, board));
        #[cfg(feature = "clock")]
        if let Some(clock) = &mut self.clock {
          draw_clock(ctx, clock, self.flipped);
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
        Screen::Game(board) => draw_game(self, ctx, &board.clone()),
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

    #[cfg(all(feature = "music", feature = "clock"))]
    if let Some(player) = &mut self.audio_engine {
      player.set_clock_bonus(get_clock_drama(&mut self.clock));
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
      gui.moved = None;
      gui.undo = Vec::new();
      #[cfg(feature = "music")]
      if let Some(ref mut player) = gui.audio_engine {
        player.clear_dramatic();
      }
    }
    Screen::Help => {
      gui.selected = None;
      gui.moved = None;
    }
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

fn render_board(
  gui: &mut LibertyChessGUI,
  ctx: &Context,
  ui: &mut Ui,
  gamestate: &Board,
  clickable: bool,
  flipped: bool,
) {
  let available_size = ctx.available_rect().size();
  let rows = gamestate.height();
  let cols = gamestate.width();
  let rows_float = rows as f32;
  let cols_float = cols as f32;
  let row_size = (available_size.y / rows_float).floor();
  let column_size = (available_size.x / cols_float).floor();
  let size = f32::max(1.0, f32::min(row_size, column_size));
  let sense = if clickable {
    Sense::click_and_drag()
  } else {
    Sense::hover()
  };
  let rect = Vec2 {
    x: size * cols_float,
    y: size * rows_float,
  };
  let (response, painter) = ui.allocate_painter(rect, sense);
  let board_rect = response.rect;
  painter.rect_filled(board_rect, Rounding::none(), Colours::WhiteSquare.value());
  if let Some(location) = response.interact_pointer_pos() {
    let hover = get_hovered(board_rect, location, size as usize, flipped, gamestate);
    register_response(gui, gamestate, &response, hover);
  }
  let (dragged, offset) = if let Some((coords, offset)) = gui.drag {
    (Some(coords), offset)
  } else {
    (None, Pos2::default())
  };
  let mut dragged_image = None;
  let mut images = Vec::new();
  for i in (0..rows).rev() {
    let (min_y, max_y) = (i as f32, (i + 1) as f32);
    let (min_y, max_y) = if flipped {
      (
        min_y.mul_add(size, board_rect.min.y),
        max_y.mul_add(size, board_rect.min.y),
      )
    } else {
      (
        max_y.mul_add(-size, board_rect.max.y),
        min_y.mul_add(-size, board_rect.max.y),
      )
    };
    for j in 0..cols {
      let coords = (i, j);
      let black_square = (i + j) % 2 == 0;
      let mut colour = if black_square {
        Colours::BlackSquare
      } else {
        Colours::WhiteSquare
      };
      if gamestate.attacked_kings().contains(&&coords) {
        colour = Colours::Check;
      } else if let Some([from, to]) = gui.moved {
        if coords == from || coords == to {
          colour = Colours::Moved;
        }
      }
      let rect = Rect {
        min: pos2((j as f32).mul_add(size, board_rect.min.x), min_y),
        max: pos2(((j + 1) as f32).mul_add(size, board_rect.min.x), max_y),
      };
      let piece = gamestate.get_piece(coords);
      let (selected, piece_rect) = if let Some(dragged) = dragged {
        let mut rect = rect;
        if dragged == coords {
          rect = rect.translate(offset.to_vec2());
          let center = rect.center().clamp(board_rect.min, board_rect.max);
          rect.set_center(center);
        }
        (Some(dragged), rect)
      } else {
        (gui.selected, rect)
      };
      if let Some(start) = selected {
        if start == coords {
          colour = Colours::Selected;
        } else if gamestate.check_pseudolegal(start, coords)
          && gamestate.get_legal(start, coords).is_some()
        {
          colour = if piece == 0 {
            if black_square {
              Colours::ValidBlack
            } else {
              Colours::ValidWhite
            }
          } else if black_square {
            Colours::ThreatenedBlack
          } else {
            Colours::ThreatenedWhite
          }
        }
      }
      if colour != Colours::WhiteSquare {
        painter.rect_filled(rect, Rounding::none(), colour.value());
      }
      if piece != 0 {
        let texture = gui.get_image(ctx, piece, size as u32);
        let image = Shape::image(texture, piece_rect, UV, Color32::WHITE);
        if dragged == Some(coords) {
          dragged_image = Some(image);
        } else {
          images.push(image);
        }
      };
    }
  }
  painter.extend(images);
  if let Some(image) = dragged_image {
    painter.add(image);
  }
}

fn get_hovered(board_rect: Rect, location: Pos2, size: usize, flipped: bool, gamestate: &Board) -> Option<((usize, usize), i8)> {
    if board_rect.contains(location) {
      let x = (location.x - board_rect.min.x) as usize / size;
      let y = if flipped {
        location.y - board_rect.min.y
      } else {
        board_rect.max.y - location.y
      } as usize
        / size;
      let coords = (y, x);
      gamestate.fetch_piece(coords).map(|piece| (coords, *piece))
    } else {
      None
    }
}

fn register_response(
  gui: &mut LibertyChessGUI,
  gamestate: &Board,
  response: &Response,
  hover: Option<((usize, usize), Piece)>,
) {
  if let Some((coords, piece)) = hover {
    let capture = piece != 0;
    let valid_piece = capture && gamestate.to_move() == (piece > 0);
    if response.clicked() {
      if let Some(selected) = gui.selected {
        attempt_move(
          gui,
          gamestate,
          selected,
          coords,
          #[cfg(feature = "sound")]
          capture,
        );
      } else if valid_piece {
        gui.selected = Some(coords);
      }
    }
    if response.drag_started() && response.dragged_by(PointerButton::Primary) && valid_piece {
      gui.drag = Some((coords, Pos2::default()));
    }
  }
  if let Some((start, ref mut offset)) = gui.drag {
    *offset += response.drag_delta();
    if response.drag_released() {
      #[cfg(feature = "sound")]
      if let Some((coords, piece)) = hover {
        if start != coords {
          let capture = piece != 0;
          attempt_move(gui, gamestate, start, coords, capture);
        }
      }
      #[cfg(not(feature = "sound"))]
      if let Some((coords, _)) = hover {
        if start != coords {
          attempt_move(gui, gamestate, start, coords);
        }
      }
      gui.drag = None;
    }
  }
}

fn attempt_move(
  gui: &mut LibertyChessGUI,
  gamestate: &Board,
  selected: (usize, usize),
  coords: (usize, usize),
  #[cfg(feature = "sound")] capture: bool,
) {
  #[cfg(feature = "sound")]
  let mut effect = Effect::Illegal;
  if gamestate.check_pseudolegal(selected, coords) {
    if let Some(mut newstate) = gamestate.get_legal(selected, coords) {
      if !newstate.promotion_available() {
        newstate.update();
        #[cfg(feature = "clock")]
        if let Some(clock) = &mut gui.clock {
          clock.switch_clocks();
        }
      }
      gui.undo.push(gamestate.clone());
      #[cfg(feature = "sound")]
      {
        effect = match newstate.state() {
          Gamestate::Checkmate(_) => Effect::Victory,
          Gamestate::Stalemate | Gamestate::Repetition | Gamestate::Move50 => Effect::Draw,
          Gamestate::InProgress => {
            if newstate.attacked_kings().is_empty() {
              if capture {
                Effect::Capture
              } else {
                Effect::Move
              }
            } else {
              Effect::Check
            }
          }
        };
      }
      #[cfg(feature = "music")]
      {
        let dramatic = get_dramatic(&newstate) + if capture { 0.5 } else { 0.0 };
        if let Some(ref mut player) = gui.audio_engine {
          player.set_dramatic(dramatic);
        }
      }
      gui.screen = Screen::Game(Box::new(newstate));
      gui.moved = Some([selected, coords]);
    }
  }
  #[cfg(feature = "sound")]
  if let Some(player) = &mut gui.audio_engine {
    player.play(&effect);
  }
  gui.selected = None;
}

// draw main areas for each screen
fn draw_menu(gui: &mut LibertyChessGUI, _ctx: &Context, ui: &mut Ui) {
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
  ComboBox::from_id_source("Gamemode")
    .selected_text("Gamemode: ".to_owned() + &gui.gamemode.to_string())
    .show_ui(ui, |ui| {
      for gamemode in all::<Presets>() {
        ui.selectable_value(
          &mut gui.gamemode,
          GameMode::Preset(gamemode),
          gamemode.to_string(),
        );
      }
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
            gui.clock = Some(Clock::new(gui.clock_data, board.to_move()));
          }
        }
        if gui.friendly {
          board.friendly_fire = true;
        }
        #[cfg(feature = "music")]
        if let Some(ref mut player) = gui.audio_engine {
          player.set_dramatic(get_dramatic(&board));
        }
        switch_screen(gui, Screen::Game(Box::new(board)));
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
  draw_clock_edit(gui, ui, size * 2.0);
}

fn draw_game(gui: &mut LibertyChessGUI, ctx: &Context, board: &Board) {
  #[cfg(feature = "clock")]
  let mut clickable;
  #[cfg(not(feature = "clock"))]
  let clickable;
  clickable = !board.promotion_available() && board.state() == Gamestate::InProgress;
  #[cfg(feature = "clock")]
  if let Some(clock) = &gui.clock {
    if clock.is_flagged() {
      gui.selected = None;
      clickable = false;
    }
  }
  Area::new("Board")
    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
    .show(ctx, |ui| {
      render_board(gui, ctx, ui, board, clickable, gui.flipped);
    });
}

fn draw_help(gui: &mut LibertyChessGUI, ctx: &Context) {
  gui.selected = Some(gui.help_page.selected());
  gui.moved = gui.help_page.moved();
  Area::new("Board")
    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
    .show(ctx, |ui| {
      render_board(gui, ctx, ui, &gui.help_page.board(), false, false);
    });
}

fn draw_settings(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui) {
  let mut new_theme = gui.config.get_theme();
  menu_button(gui, ui);
  ComboBox::from_id_source("Theme")
    .selected_text("Theme: ".to_owned() + &new_theme.show())
    .show_ui(ui, |ui| {
      for theme in all::<PresetTheme>() {
        ui.selectable_value(&mut new_theme, Theme::Preset(theme), theme.to_string());
      }
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
  ui.add(Slider::new(&mut size, 16..=36).text("Font size"));
  if size != gui.config.get_text_size() {
    gui.config.set_text_size(ctx, size);
  }
  #[cfg(feature = "sound")]
  {
    let mut sound = gui.audio_engine.is_some();
    if checkbox(ui, &mut sound, "Sound", None) {
      gui.audio_engine = if sound { Engine::new() } else { None }
    };
    if let Some(ref mut engine) = gui.audio_engine {
      let mut volume = engine.get_sound_volume();
      ui.add(Slider::new(&mut volume, 0..=DEFAULT_VOLUME).text("Move Volume"));
      if volume != engine.get_sound_volume() {
        engine.set_sound_volume(volume);
      }
      #[cfg(feature = "music")]
      {
        let mut music = engine.music_enabled();
        if checkbox(ui, &mut music, "Music", Some(engine)) {
          engine.toggle_music();
        }
        if music {
          let mut dramatic = engine.dramatic_enabled();
          if checkbox(ui, &mut dramatic, "Dramatic Music", Some(engine)) {
            engine.toggle_dramatic();
          }
          let mut volume = engine.get_music_volume();
          ui.add(Slider::new(&mut volume, 0..=DEFAULT_VOLUME).text("Music Volume"));
          if volume != engine.get_music_volume() {
            engine.set_music_volume(volume);
          }
        }
      }
    }
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
    gui.moved = None;
    #[cfg(feature = "clock")]
    if let Some(clock) = &mut gui.clock {
      clock.switch_clocks();
    };
  }

  #[cfg(feature = "clock")]
  if let Some(clock) = &mut gui.clock {
    if !clock.is_flagged() {
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
        for piece in promotion.iter() {
          ui.selectable_value(&mut gui.promotion, *piece, to_name(*piece));
        }
      });
    if ui.button("Promote").clicked() {
      gamestate.promote(gui.promotion);
      gui.screen = Screen::Game(gamestate.clone());
      #[cfg(feature = "clock")]
      if let Some(clock) = &mut gui.clock {
        clock.switch_clocks();
      }
    }
  }

  // let the user copy the FEN to clipboard
  if ui.button("Copy FEN").clicked() {
    ui.output().copied_text = get_fen(gui);
  }

  // if the game is over, report the reason
  let state = gamestate.state();
  if state != Gamestate::InProgress {
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
      Gamestate::InProgress => unreachable!(),
    });
  }
}

// general helper functions

#[cfg(feature = "music")]
fn get_dramatic(board: &Board) -> f64 {
  let mut dramatic = 0.0;
  if board.state() != Gamestate::InProgress {
    dramatic += 0.5;
  }
  if !board.attacked_kings().is_empty() {
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

#[cfg(target_arch = "wasm32")]
fn main() {
  spawn_local(async {
    eframe::start_web(
      "Liberty Chess",
      eframe::WebOptions::default(),
      Box::new(|cc| Box::new(LibertyChessGUI::new(cc))),
    )
    .await
    .expect("Wasm failed to load");
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
  );
}
