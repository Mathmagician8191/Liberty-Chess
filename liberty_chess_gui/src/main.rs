use crate::colours::Colours;
use crate::credits::Credits;
use crate::gamemodes::{GameMode, Presets};
use crate::help_page::HelpPage;
use crate::themes::Theme;
use eframe::egui;
use egui::{
  Align2, Area, Button, ColorImage, ComboBox, Context, Image, RichText, SidePanel, Slider,
  TextStyle, TextureFilter, TopBottomPanel, Ui, Vec2,
};
use enum_iterator::all;
use liberty_chess::{print_secs, to_name, Board, Clock, Gamestate, Piece};
use resvg::usvg::{FitTo, Tree};
use std::time::Duration;
use tiny_skia::{Pixmap, Transform};

#[cfg(feature = "benchmarking")]
use std::time::Instant;

#[cfg(feature = "clock")]
use liberty_chess::Type;

#[cfg(feature = "clipboard")]
use clipboard::ClipboardProvider;

#[cfg(feature = "sound")]
use soloud::{Soloud, Wav};

// enums in own file
mod colours;
mod credits;
mod gamemodes;
mod help_page;
mod themes;

// files to load resources
mod images;
#[cfg(feature = "sound")]
mod sound;

//sizes of things
const ICON_SIZE: u32 = 48;
#[allow(clippy::cast_precision_loss)]
const ICON_SIZE_FLOAT: f32 = ICON_SIZE as f32;
const DEFAULT_TEXT_SIZE: u8 = 24;
#[cfg(feature = "sound")]
const DEFAULT_VOLUME: u8 = 100;

enum Screen {
  Menu,
  Game,
  Help,
  Credits,
  Settings,
}

struct LibertyChessGUI {
  // current screen
  screen: Screen,

  // global settings
  theme: Theme,
  text_size: u8,

  // fields for board rendering
  gamestate: Option<Board>,
  selected: Option<(usize, usize)>,
  moved: Option<[(usize, usize); 2]>,

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
  clock: Option<Clock>,
  promotion: Piece,
  #[cfg(feature = "clipboard")]
  clipboard: Option<clipboard::ClipboardContext>,

  // field for help screen
  help_page: HelpPage,

  // field for credits
  credits: Credits,

  //sound players and audio
  #[cfg(feature = "sound")]
  effect_player: Option<Soloud>,
  #[cfg(feature = "sound")]
  volume: u8,
  #[cfg(feature = "sound")]
  audio: [Wav; 2],

  // images and a render cache - used on game screen
  images: [Tree; 36],
  renders: [Option<egui::TextureHandle>; 37],

  // for measuring FPS
  #[cfg(feature = "benchmarking")]
  instant: Instant,
  #[cfg(feature = "benchmarking")]
  frames: u32,
  #[cfg(feature = "benchmarking")]
  seconds: u64,
}

impl LibertyChessGUI {
  fn new(ctx: &eframe::CreationContext) -> Self {
    let theme;
    let text_size;
    #[cfg(feature = "sound")]
    let sound;
    #[cfg(feature = "sound")]
    let volume;
    let gamestate;
    if let Some(data) = ctx.storage {
      theme = themes::get_theme(data.get_string("Theme"));
      text_size = load_data(data.get_string("TextSize"), DEFAULT_TEXT_SIZE);
      gamestate = data
        .get_string("Board")
        .as_ref()
        .and_then(|fen| Board::new(fen).ok());
      #[cfg(feature = "sound")]
      {
        sound = data.get_string("Sound") != Some("false".to_string());
        volume = load_data(data.get_string("Volume"), DEFAULT_VOLUME);
      }
    } else {
      // set up default parameters
      theme = Theme::Dark;
      text_size = DEFAULT_TEXT_SIZE;
      gamestate = None;
      #[cfg(feature = "sound")]
      {
        sound = true;
        volume = DEFAULT_VOLUME;
      }
    };
    set_style(&ctx.egui_ctx, text_size);
    ctx.egui_ctx.set_visuals(theme.get_visuals());
    Self {
      screen: if gamestate.is_some() {
        Screen::Game
      } else {
        Screen::Menu
      },

      theme,
      text_size,

      gamestate,
      selected: None,
      moved: None,

      gamemode: GameMode::Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      friendly: false,
      message: None,
      #[cfg(feature = "clock")]
      clock_type: Type::None,
      #[cfg(feature = "clock")]
      clock_data: [10; 4],

      undo: Vec::new(),
      clock: None,
      promotion: liberty_chess::QUEEN,
      #[cfg(feature = "clipboard")]
      clipboard: ClipboardProvider::new().ok(),

      help_page: HelpPage::PawnForward,
      credits: Credits::Coding,

      #[cfg(feature = "sound")]
      effect_player: sound::get_player(sound),
      #[cfg(feature = "sound")]
      volume,
      #[cfg(feature = "sound")]
      audio: sound::get(),

      images: images::get(),
      renders: [(); 37].map(|_| None),

      #[cfg(feature = "benchmarking")]
      instant: Instant::now(),
      #[cfg(feature = "benchmarking")]
      frames: 0,
      #[cfg(feature = "benchmarking")]
      seconds: 0,
    }
  }

  fn get_image(&mut self, ctx: &Context, piece: Piece, size: u32) -> egui::TextureId {
    let index = match piece {
      _ if piece > 0 => (piece - 1) as usize,
      _ if piece < 0 => (17 - piece) as usize,
      _ => {
        if let Some(map) = &self.renders[36] {
          if map.size() == [size as usize; 2] {
            return map.id();
          }
        }
        let texture = ctx.load_texture(
          "square",
          ColorImage::new([size as usize; 2], egui::Color32::from_black_alpha(0)),
          TextureFilter::Nearest,
        );
        self.renders[36] = Some(texture.clone());
        return texture.id();
      }
    };
    if let Some(map) = &self.renders[index] {
      if map.size() == [size as usize; 2] {
        return map.id();
      }
    }
    let mut pixmap = Pixmap::new(size, size).unwrap();
    resvg::render(
      &self.images[index],
      FitTo::Size(size, size),
      Transform::default(),
      pixmap.as_mut(),
    )
    .unwrap();
    let image = ColorImage::from_rgba_unmultiplied([size as usize; 2], pixmap.data());
    let texture = ctx.load_texture("piece", image, TextureFilter::Nearest);
    self.renders[index] = Some(texture.clone());
    texture.id()
  }
}

impl eframe::App for LibertyChessGUI {
  fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
    match &self.screen {
      Screen::Game => {
        SidePanel::right("Sidebar")
          .resizable(false)
          .show(ctx, |ui| draw_game_sidebar(self, ui));
        if let Some(clock) = &mut self.clock {
          draw_clock(ctx, clock);
        }
      }
      Screen::Help => {
        SidePanel::left("Help menu")
          .resizable(false)
          .show(ctx, |ui| {
            menu_button(self, ui);
            egui::ScrollArea::vertical().show(ui, |ui| {
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
            ui.add(egui::Label::new("Credits:").wrap(false));
            for page in all::<Credits>() {
              if ui.add(Button::new(page.title()).wrap(false)).clicked() {
                self.credits = page;
              }
            }
          });
      }
      Screen::Menu | Screen::Settings => (),
    };

    egui::CentralPanel::default().show(ctx, |ui| {
      match &self.screen {
        Screen::Menu => draw_menu(self, ctx, ui),
        Screen::Game => draw_game(self, ctx),
        Screen::Help => draw_help(self, ctx),
        Screen::Credits => draw_credits(self, ctx, ui),
        Screen::Settings => {
          Area::new("Settings")
            .anchor(Align2::CENTER_TOP, Vec2::ZERO)
            .show(ctx, |ui| draw_settings(self, ctx, ui));
        }
      };
    });
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
      ctx.request_repaint_after(Duration::ZERO);
    }
    #[cfg(not(feature = "benchmarking"))]
    {
      ctx.request_repaint_after(Duration::from_millis(200));
    }
  }

  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    storage.set_string("Theme", self.theme.to_string());
    storage.set_string("TextSize", self.text_size.to_string());
    let fen = if let Some(gamestate) = &self.gamestate {
      if gamestate.promotion_available() {
        // should always be some
        self
          .undo
          .last()
          .map_or_else(String::new, ToString::to_string)
      } else {
        gamestate.to_string()
      }
    } else {
      String::new()
    };
    storage.set_string("Board", fen);
    #[cfg(feature = "sound")]
    {
      storage.set_string("Sound", self.effect_player.is_some().to_string());
      storage.set_string("Volume", self.volume.to_string());
    }
  }
}

fn switch_screen(gui: &mut LibertyChessGUI, screen: Screen) {
  match &gui.screen {
    Screen::Menu => gui.message = None,
    Screen::Game => {
      gui.gamestate = None;
      gui.selected = None;
      gui.moved = None;
      gui.undo = Vec::new();
    }
    Screen::Help => {
      gui.selected = None;
      gui.moved = None;
    }
    Screen::Credits | Screen::Settings => (),
  }
  gui.screen = screen;
}

fn render_board(
  gui: &mut LibertyChessGUI,
  ctx: &Context,
  ui: &mut Ui,
  gamestate: &Board,
  clickable: bool,
) {
  let available_size = ctx.available_rect().size();
  let rows = gamestate.height();
  let columns = gamestate.width();
  let row_size = (available_size.y / (rows as f32)).floor();
  let column_size = (available_size.x / (columns as f32)).floor();
  let size = f32::max(1.0, f32::min(row_size, column_size));
  egui::Grid::new("Board")
    .num_columns(columns)
    .spacing([0.0; 2])
    .min_col_width(size)
    .min_row_height(size)
    .show(ui, |ui| {
      for i in (0..rows).rev() {
        for j in 0..columns {
          let coords = (i, j);
          let black_square = (i + j) % 2 == 0;
          let piece = gamestate.get_piece(coords);
          let mut colour = if black_square {
            Colours::BlackSquare
          } else {
            Colours::WhiteSquare
          };
          if let Some([from, to]) = gui.moved {
            if coords == from || coords == to {
              colour = Colours::Moved;
            }
          }
          if gamestate.attacked_kings().contains(&&coords) {
            colour = Colours::Check;
          }
          if let Some(start) = gui.selected {
            if gamestate.check_pseudolegal(start, coords)
              && gamestate.get_legal(start, coords).is_some()
            {
              colour = if gamestate.get_piece(coords) == 0 {
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
          if Some(coords) == gui.selected {
            colour = Colours::Selected;
          }
          let texture = gui.get_image(ctx, piece, size as u32);
          let icon = Image::new(texture, [size; 2]).bg_fill(colour.value());
          let response = ui.add(icon).interact(egui::Sense::click());
          if clickable && response.clicked() {
            if let Some(selected) = gui.selected {
              if let Some(gamestate) = &mut gui.gamestate {
                if gamestate.check_pseudolegal(selected, coords) {
                  if let Some(mut newstate) = gamestate.get_legal(selected, coords) {
                    if !newstate.promotion_available() {
                      newstate.update();
                      if let Some(clock) = &mut gui.clock {
                        clock.switch_clocks();
                      }
                    }
                    gui.undo.push(gamestate.clone());
                    #[cfg(feature = "sound")]
                    if let Some(player) = &mut gui.effect_player {
                      player.set_global_volume(f32::from(gui.volume) / 100.0);
                      player.play(&gui.audio[usize::from(gamestate.get_piece(coords) != 0)]);
                    }
                    gui.gamestate = Some(newstate);
                    gui.moved = Some([selected, coords]);
                  }
                }
              }
              gui.selected = None;
            } else {
              let piece = gamestate.get_piece(coords);
              if piece != 0 && gamestate.to_move() == (piece > 0) {
                gui.selected = Some(coords);
              }
            }
          }
        }
        ui.end_row();
      }
    });
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
    .selected_text("Gamemode: ".to_string() + &gui.gamemode.to_string())
    .show_ui(ui, |ui| {
      for gamemode in all::<Presets>() {
        ui.selectable_value(
          &mut gui.gamemode,
          GameMode::Preset(gamemode),
          gamemode.to_string(),
        );
      }
      ui.selectable_value(&mut gui.gamemode, GameMode::Custom, "Custom")
    });
  if let GameMode::Preset(preset) = gui.gamemode {
    gui.fen = preset.value();
  } else {
    text_edit(ui, 11.5, 220.0, &mut gui.fen);
  }
  ui.checkbox(&mut gui.friendly, "Friendly Fire");
  if ui.button("Start Game").clicked() {
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
        gui.gamestate = Some(board);
        switch_screen(gui, Screen::Game);
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
  {
    ComboBox::from_id_source("Clock")
      .selected_text(gui.clock_type.to_string())
      .show_ui(ui, |ui| {
        for clock_type in all::<Type>() {
          ui.selectable_value(&mut gui.clock_type, clock_type, clock_type.to_string());
        }
      });
    match gui.clock_type {
      Type::None => (),
      Type::Increment => {
        ui.horizontal_top(|ui| {
          let mut input: Vec<String> = gui.clock_data.iter().map(u64::to_string).collect();
          ui.label("Time (minutes):".to_string());
          text_edit(ui, 0.0, 40.0, &mut input[0]);
          ui.label("Increment (seconds):");
          text_edit(ui, 0.0, 40.0, &mut input[2]);
          if let Ok(value) = input[0].parse::<u64>() {
            let value = u64::min(value, 1440);
            gui.clock_data[0] = value;
            gui.clock_data[1] = value;
          }
          if let Ok(value) = input[2].parse::<u64>() {
            let value = u64::min(value, 1440);
            gui.clock_data[2] = value;
            gui.clock_data[3] = value;
          }
        });
      }
      Type::Handicap => {
        let mut input: Vec<String> = gui.clock_data.iter().map(u64::to_string).collect();
        ui.horizontal_top(|ui| {
          ui.label("White Time (minutes):");
          text_edit(ui, 0.0, 40.0, &mut input[0]);
          ui.label("White Increment (seconds):");
          text_edit(ui, 0.0, 40.0, &mut input[2]);
        });
        ui.horizontal_top(|ui| {
          ui.label("Black Time (minutes):");
          text_edit(ui, 0.0, 40.0, &mut input[1]);
          ui.label("Black Increment (seconds):");
          text_edit(ui, 0.0, 40.0, &mut input[3]);
        });
        for (i, value) in input.iter().enumerate() {
          if let Ok(value) = value.parse::<u64>() {
            gui.clock_data[i] = u64::min(value, 1440);
          }
        }
      }
    }
  }
}

fn draw_game(gui: &mut LibertyChessGUI, ctx: &Context) {
  let gamestate = gui.gamestate.clone().expect("No board despite game");
  let mut clickable =
    !gamestate.promotion_available() && gamestate.state() == Gamestate::InProgress;
  if let Some(clock) = &gui.clock {
    if clock.is_flagged() {
      gui.selected = None;
      clickable = false;
    }
  }
  Area::new("Board")
    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
    .show(ctx, |ui| render_board(gui, ctx, ui, &gamestate, clickable));
}

fn draw_help(gui: &mut LibertyChessGUI, ctx: &Context) {
  gui.selected = Some(gui.help_page.selected());
  gui.moved = gui.help_page.moved();
  Area::new("Board")
    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
    .show(ctx, |ui| {
      render_board(gui, ctx, ui, &gui.help_page.board(), false);
    });
}

fn draw_credits(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui) {
  match gui.credits {
    Credits::Coding => {
      ui.label("Programming done by:");
      github(ui, "Mathmagician8191");
      ui.label("The code is licensed under GPL v3 and can be found here:");
      let code_link = "https://github.com/Mathmagician8191/Liberty-Chess".to_string();
      link(ui, code_link.clone(), code_link);
    }
    Credits::Images => {
      egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label("Image credit by license");
        ui.label("\nCC-BY-SA 3.0:");
        ui.label("Apathor:");
        get_row(gui, ctx, ui, "NnBbRr");
        ui.label("TomFryers:");
        get_row(gui, ctx, ui, "PpQqKk");
        wikipedia(ui, "Cburnett");
        get_row(gui, ctx, ui, "AaCc");
        wikimedia(ui, "Francois-Pier", "Francois-Pier");
        get_row(gui, ctx, ui, "Ll");
        wikimedia(ui, "NikNaks", "NikNaks");
        get_row(gui, ctx, ui, "Hh");
        link(ui, "greenchess.net", "https://greenchess.net".to_string());
        get_row(gui, ctx, ui, "IiMmOoWw");
        ui.label("\nCC-BY-SA 4.0:");
        wikimedia(ui, "Sunny3113", "Sunny3113");
        get_row(gui, ctx, ui, "ZzXxU");
        wikimedia(ui, "Iago Casabiell González", "Iagocasabiell");
        get_row(gui, ctx, ui, "Ee");
        ui.label("\nCC0:");
        wikipedia(ui, "CheChe");
        ui.add(get_icon(gui, ctx, 'u'));
      });
    }
    #[cfg(feature = "sound")]
    Credits::Sound => {
      ui.label("The sound effects for piece moving were done by:");
      github(ui, "Enigmahack");
      ui.label("They are licensed under AGPLv3+");
    }
  }
}

fn draw_settings(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui) {
  let theme = gui.theme;
  menu_button(gui, ui);
  ComboBox::from_id_source("Theme")
    .selected_text("Theme: ".to_string() + &gui.theme.to_string())
    .show_ui(ui, |ui| {
      for theme in all::<Theme>() {
        ui.selectable_value(&mut gui.theme, theme, theme.to_string());
      }
    });
  #[cfg(feature = "sound")]
  {
    let mut sound = gui.effect_player.is_some();
    ui.checkbox(&mut sound, "Sound");
    if sound == gui.effect_player.is_none() {
      gui.effect_player = sound::get_player(sound);
    }
    ui.add(Slider::new(&mut gui.volume, 0..=100).text("Volume"));
  }
  let size = gui.text_size;
  ui.add(Slider::new(&mut gui.text_size, 16..=36).text("Font size"));
  if size != gui.text_size {
    set_style(ctx, gui.text_size);
  }
  if gui.theme != theme {
    ctx.set_visuals(gui.theme.get_visuals());
  }
}

// draw areas for specific screens

fn draw_clock(ctx: &Context, clock: &mut Clock) {
  clock.update();
  let (white, black) = clock.get_clocks();
  let mut white_text = RichText::new(print_secs(white.as_secs()));
  let mut black_text = RichText::new(print_secs(black.as_secs()));
  let color = if clock.is_flagged() {
    Colours::Check
  } else {
    Colours::Selected
  };
  if clock.to_move() {
    white_text = white_text.color(color.value());
  } else {
    black_text = black_text.color(color.value());
  }
  TopBottomPanel::bottom("White Clock")
    .resizable(false)
    .show(ctx, |ui| ui.label(white_text));
  TopBottomPanel::top("Black Clock")
    .resizable(false)
    .show(ctx, |ui| ui.label(black_text));
}

fn draw_game_sidebar(gui: &mut LibertyChessGUI, ui: &mut Ui) {
  menu_button(gui, ui);
  if !gui.undo.is_empty() && ui.button("Undo").clicked() {
    gui.gamestate = gui.undo.pop();
    gui.moved = None;
    if let Some(clock) = &mut gui.clock {
      clock.switch_clocks();
    }
  }

  // display promotion if applicable
  if let Some(gamestate) = &mut gui.gamestate {
    if gamestate.promotion_available() {
      let promotion = gamestate.promotion_options();
      if !promotion.is_empty() {
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
          if let Some(clock) = &mut gui.clock {
            clock.switch_clocks();
          }
        }
      }
    }

    // let the user copy the FEN to clipboard
    #[cfg(feature = "clipboard")]
    if let Some(clipboard) = &mut gui.clipboard {
      if ui.button("Copy FEN to clipboard").clicked() {
        clipboard.set_contents(gamestate.to_string()).unwrap();
      }
    }

    // if the game is over, report the reason
    let state = gamestate.state();
    if state != Gamestate::InProgress {
      ui.label(match state {
        Gamestate::Checkmate(bool) => {
          if bool {
            "Black wins by checkmate"
          } else {
            "White wins by checkmate"
          }
        }
        Gamestate::Stalemate => "Draw by stalemate",
        Gamestate::Move50 => "Draw by 50 move rule",
        Gamestate::Repetition => "Draw by 3-fold repetition",
        Gamestate::InProgress => unreachable!(),
      });
    }
  }
}

// general helper functions

fn menu_button(gui: &mut LibertyChessGUI, ui: &mut Ui) {
  if ui.button("Menu").clicked() {
    switch_screen(gui, Screen::Menu);
  }
}

fn get_row(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui, pieces: &str) {
  ui.horizontal_wrapped(|ui| {
    for c in pieces.chars() {
      ui.add(get_icon(gui, ctx, c));
    }
  });
}

fn text_edit(ui: &mut Ui, char_size: f32, min_size: f32, string: &mut String) {
  let space = f32::min(
    ui.available_size().x,
    f32::max(char_size * string.len() as f32, min_size),
  );
  ui.add_sized([space, 0.0], egui::TextEdit::singleline(string));
}

fn get_icon(gui: &mut LibertyChessGUI, ctx: &Context, piece: char) -> Image {
  Image::new(
    gui.get_image(ctx, liberty_chess::to_piece(piece).unwrap(), ICON_SIZE),
    [ICON_SIZE_FLOAT, ICON_SIZE_FLOAT],
  )
}

// convenient wrappers for links

fn github(ui: &mut Ui, name: &str) {
  link(ui, name, "https://github.com/".to_string() + name);
}

fn wikipedia(ui: &mut Ui, name: &str) {
  link(
    ui,
    name.to_string() + ":",
    "https://en.wikipedia.org/wiki/User:".to_string() + name,
  );
}

fn wikimedia(ui: &mut Ui, name: &str, username: &str) {
  link(
    ui,
    name.to_string() + ":",
    "https://commons.wikimedia.org/wiki/User:".to_string() + username,
  );
}

fn link(ui: &mut Ui, name: impl Into<egui::WidgetText>, link: String) {
  ui.add(egui::Hyperlink::from_label_and_url(name, link));
}

// configuration functions

fn set_style(ctx: &Context, size: u8) {
  let mut style = (*ctx.style()).clone();
  let font = egui::FontId::new(f32::from(size), egui::FontFamily::Proportional);
  style.text_styles = [
    (TextStyle::Body, font.clone()),
    (TextStyle::Button, font.clone()),
    (TextStyle::Monospace, font),
  ]
  .into();
  ctx.set_style(style);
}

fn load_data<T: std::str::FromStr>(data: Option<String>, default: T) -> T {
  if let Some(data) = data {
    data.parse::<T>().unwrap_or(default)
  } else {
    default
  }
}

#[cfg(target_arch = "wasm32")]
fn main() {
  eframe::start_web(
    "Liberty Chess",
    eframe::WebOptions::default(),
    Box::new(|cc| Box::new(LibertyChessGUI::new(cc))),
  )
  .expect("Wasm failed to load");
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
  let size = ICON_SIZE;
  let mut pixmap = Pixmap::new(size, size).unwrap();
  resvg::render(
    &images::get()[11],
    FitTo::Size(size, size),
    Transform::default(),
    pixmap.as_mut(),
  )
  .unwrap();
  let options = eframe::NativeOptions {
    // Disable vsync when benchmarking to remove the framerate limit
    vsync: !cfg!(feature = "benchmarking"),
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
