use crate::colours::Colours;
use crate::credits::Credits;
use crate::gamemodes::{GameMode, Presets};
use crate::help_page::HelpPage;
use crate::themes::Theme;
use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use egui::widgets::Hyperlink;
use egui::{
  Color32, ColorImage, ComboBox, Context, FontFamily, FontId, Image, RichText, TextStyle,
  TextureFilter, TextureHandle, Ui,
};
use enum_iterator::all;
use liberty_chess::{print_secs, to_name, Board, Clock, Gamestate, Piece, Type};
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;

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
mod sound;

const MENU_TEXT: &str = "Back to Menu";

//sizes of things
const ICON_SIZE: usize = 48;
const TEXT_SIZE: f32 = 24.0;
const SMALL_SIZE: f32 = 16.0;

enum Screen {
  Menu,
  Game,
  Help,
  Credits,
}

struct LibertyChessGUI {
  // current screen
  screen: Screen,

  // global theme
  theme: Theme,

  // fields for board rendering
  gamestate: Option<Board>,
  selected: Option<(usize, usize)>,
  moved: Option<[(usize, usize); 2]>,

  // fields for main menu
  fen: String,
  gamemode: GameMode,
  message: Option<String>,
  clock_type: Type,
  clock_data: [u64; 4],

  // fields for game screen
  undo: Vec<Board>,
  clock: Option<Clock>,
  promotion: Piece,
  clipboard: Option<ClipboardContext>,

  // field for help screen
  help_page: HelpPage,

  // field for credits
  credits: Credits,

  //sound players and audio
  #[cfg(feature = "sound")]
  effect_player: Option<Soloud>,
  #[cfg(feature = "sound")]
  audio: [Wav; 2],

  // images and a render cache - used on game screen
  images: [usvg::Tree; 36],
  renders: [Option<TextureHandle>; 37],

  // for measuring FPS
  instant: Instant,
  frames: u32,
  seconds: u64,
}

impl LibertyChessGUI {
  fn new(ctx: &Context) -> Self {
    let mut style = (*ctx.style()).clone();
    let font = FontId::new(TEXT_SIZE, FontFamily::Proportional);
    style.text_styles = [
      (TextStyle::Heading, font.clone()),
      (
        TextStyle::Body,
        FontId::new(SMALL_SIZE, FontFamily::Proportional),
      ),
      (TextStyle::Button, font),
    ]
    .into();
    ctx.set_style(style);
    let theme = Theme::Dark;
    ctx.set_visuals(theme.get_visuals());
    Self {
      screen: Screen::Menu,

      theme,

      gamestate: None,
      selected: None,
      moved: None,

      gamemode: GameMode::Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      message: None,
      clock_type: Type::None,
      clock_data: [10, 10, 10, 10],

      undo: Vec::new(),
      clock: None,
      promotion: liberty_chess::QUEEN,
      clipboard: ClipboardProvider::new().ok(),

      help_page: HelpPage::PawnForward,
      credits: Credits::Coding,

      #[cfg(feature = "sound")]
      effect_player: Soloud::default().ok(),
      #[cfg(feature = "sound")]
      audio: sound::get(),

      images: images::get(),
      renders: [(); 37].map(|_| None),

      instant: Instant::now(),
      frames: 0,
      seconds: 0,
    }
  }

  fn get_image(&mut self, ctx: &Context, piece: Piece, size: usize) -> egui::TextureId {
    let index = match piece {
      _ if piece > 0 => (piece - 1) as usize,
      _ if piece < 0 => (17 - piece) as usize,
      _ => {
        if let Some(map) = &self.renders[36] {
          if map.size() == [size, size] {
            return map.id();
          }
        }
        let texture = ctx.load_texture(
          "square",
          ColorImage::new([size, size], Color32::from_black_alpha(0)),
          TextureFilter::Nearest,
        );
        self.renders[36] = Some(texture.clone());
        return texture.id();
      }
    };
    if let Some(map) = &self.renders[index] {
      if map.size() == [size, size] {
        return map.id();
      }
    }
    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    resvg::render(
      &self.images[index],
      usvg::FitTo::Size(size as u32, size as u32),
      tiny_skia::Transform::default(),
      pixmap.as_mut(),
    )
    .unwrap();
    let image = egui::ColorImage::from_rgba_unmultiplied([size, size], pixmap.data());
    let texture = ctx.load_texture("piece", image, TextureFilter::Nearest);
    self.renders[index] = Some(texture.clone());
    texture.id()
  }
}

impl eframe::App for LibertyChessGUI {
  fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
    let theme = self.theme;
    egui::TopBottomPanel::top("Topbar")
      .resizable(false)
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ComboBox::from_id_source("Theme")
            .selected_text(size(
              "Theme: ".to_string() + &self.theme.to_string(),
              SMALL_SIZE,
            ))
            .show_ui(ui, |ui| {
              for theme in all::<Theme>() {
                ui.selectable_value(&mut self.theme, theme, size(theme.to_string(), SMALL_SIZE));
              }
            });
          #[cfg(feature = "sound")]
          {
            let mut sound = self.effect_player.is_some();
            ui.checkbox(&mut sound, size("Sound", SMALL_SIZE));
            if sound == self.effect_player.is_none() {
              self.effect_player = if sound { Soloud::default().ok() } else { None }
            }
          }
        });
      });
    if self.theme != theme {
      ctx.set_visuals(self.theme.get_visuals());
    }
    match &self.screen {
      Screen::Game => {
        egui::SidePanel::right("Sidebar")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(MENU_TEXT).clicked() {
              switch_screen(self, Screen::Menu);
            }
            if !self.undo.is_empty() && ui.button("Undo").clicked() {
              self.gamestate = self.undo.pop();
              self.moved = None;
              if let Some(clock) = &mut self.clock {
                clock.switch_clocks();
              }
            }

            // display promotion if applicable
            if let Some(gamestate) = &mut self.gamestate {
              if gamestate.promotion_available() {
                let promotion = gamestate.promotion_options();
                if !promotion.is_empty() {
                  if !promotion.contains(&self.promotion) {
                    self.promotion = promotion[0];
                  }
                  ComboBox::from_id_source("Promote")
                    .selected_text(to_name(self.promotion))
                    .show_ui(ui, |ui| {
                      for piece in promotion.iter() {
                        ui.selectable_value(&mut self.promotion, *piece, to_name(*piece));
                      }
                    });
                  if ui.button("Promote").clicked() {
                    gamestate.promote(self.promotion);
                  }
                }
              }

              // let the user copy the FEN to clipboard
              if let Some(clipboard) = &mut self.clipboard {
                if ui.button("Copy FEN to clipboard").clicked() {
                  clipboard.set_contents(gamestate.to_string()).unwrap();
                }
              }

              // if the game is over, report the reason
              let state = gamestate.state();
              if state != Gamestate::InProgress {
                ui.heading(match state {
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
          });
        if let Some(clock) = &mut self.clock {
          draw_clock(ctx, clock);
        }
      }
      Screen::Help => {
        egui::SidePanel::left("Help menu")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(MENU_TEXT).clicked() {
              switch_screen(self, Screen::Menu);
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
              for page in all::<HelpPage>() {
                let mut text = RichText::new(page.title());
                if page == self.help_page {
                  text = text.color(Colours::ValidBlack.value());
                }
                if ui.button(text).clicked() {
                  self.help_page = page;
                }
              }
            });
          });
        egui::TopBottomPanel::bottom("Description")
          .resizable(false)
          .show(ctx, |ui| ui.heading(self.help_page.description()));
      }
      Screen::Credits => {
        egui::SidePanel::left("Credits menu")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(MENU_TEXT).clicked() {
              switch_screen(self, Screen::Menu);
            }
            ui.heading("Credits:");
            for page in all::<Credits>() {
              if ui.button(page.title()).clicked() {
                self.credits = page;
              }
            }
          });
      }
      Screen::Menu => (),
    };

    egui::CentralPanel::default().show(ctx, |ui| {
      match &self.screen {
        Screen::Menu => draw_menu(self, ctx, ui),
        Screen::Game => draw_game(self, ctx),
        Screen::Help => draw_help(self, ctx),
        Screen::Credits => draw_credits(self, ctx, ui),
      };
    });
    // Add no delay between rendering frames and log FPS when benchmarking
    if cfg!(feature = "benchmarking") {
      self.frames += 1;
      let duration = self.instant.elapsed().as_secs();
      if duration - self.seconds > 0 {
        self.seconds = duration;
        println!("{} FPS", self.frames);
        self.frames = 0;
      }
      ctx.request_repaint_after(Duration::ZERO);
    } else {
      ctx.request_repaint_after(Duration::from_millis(200));
    }
  }
}

fn switch_screen(gui: &mut LibertyChessGUI, screen: Screen) {
  match &gui.screen {
    Screen::Menu => gui.message = None,
    Screen::Game => {
      gui.selected = None;
      gui.moved = None;
      gui.undo = Vec::new();
    }
    Screen::Help => {
      gui.selected = None;
      gui.moved = None;
    }
    Screen::Credits => (),
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
    .spacing([0.0, 0.0])
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
          let texture = gui.get_image(ctx, piece, size as usize);
          let icon = Image::new(texture, [size, size]).bg_fill(colour.value());
          let response = ui.add(icon).interact(egui::Sense::click());
          if clickable && response.clicked() {
            if let Some(selected) = gui.selected {
              if let Some(gamestate) = &mut gui.gamestate {
                if gamestate.check_pseudolegal(selected, coords) {
                  if let Some(mut newstate) = gamestate.get_legal(selected, coords) {
                    if !newstate.promotion_available() {
                      newstate.update();
                    }
                    gui.undo.push(gamestate.clone());
                    #[cfg(feature = "sound")]
                    if let Some(player) = &gui.effect_player {
                      player.play(
                        &gui.audio[if gamestate.get_piece(coords) == 0 {
                          0
                        } else {
                          1
                        }],
                      );
                    }
                    gui.gamestate = Some(newstate);
                    gui.moved = Some([selected, coords]);
                    if let Some(clock) = &mut gui.clock {
                      clock.switch_clocks();
                    }
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
  egui::TopBottomPanel::bottom("White Clock")
    .resizable(false)
    .show(ctx, |ui| ui.heading(white_text));
  egui::TopBottomPanel::top("Black Clock")
    .resizable(false)
    .show(ctx, |ui| ui.heading(black_text));
}

fn draw_menu(gui: &mut LibertyChessGUI, _ctx: &Context, ui: &mut Ui) {
  ui.horizontal_top(|ui| {
    if ui.button("Help").clicked() {
      switch_screen(gui, Screen::Help);
    }
    if ui.button("Credits").clicked() {
      switch_screen(gui, Screen::Credits);
    }
  });
  egui::ComboBox::from_id_source("Gamemode")
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
  if ui.button("Start Game").clicked() {
    match Board::new(&gui.fen) {
      Ok(board) => {
        match gui.clock_type {
          Type::None => gui.clock = None,
          Type::Increment | Type::Handicap => {
            gui.clock = Some(Clock::new(gui.clock_data, board.to_move()));
          }
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
    ui.heading(message);
  }
  egui::ComboBox::from_id_source("Clock")
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
  egui::Area::new("Board")
    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    .show(ctx, |ui| render_board(gui, ctx, ui, &gamestate, clickable));
}

fn draw_help(gui: &mut LibertyChessGUI, ctx: &Context) {
  gui.selected = Some(gui.help_page.selected());
  gui.moved = gui.help_page.moved();
  egui::Area::new("Board")
    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    .show(ctx, |ui| {
      render_board(gui, ctx, ui, &gui.help_page.board(), false);
    });
}

fn draw_credits(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui) {
  match gui.credits {
    Credits::Coding => {
      ui.heading("Programming done by:");
      ui.add(github("Mathmagician8191"));
    }
    Credits::Images => {
      egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.heading("Image credit by license");
        ui.heading("\nCC-BY-SA 3.0");
        ui.heading("Apathor:");
        get_row(gui, ctx, ui, "NnBbRr");
        ui.heading("TomFryers:");
        get_row(gui, ctx, ui, "PpQqKk");
        ui.add(wikipedia("Cburnett"));
        get_row(gui, ctx, ui, "AaCc");
        ui.add(wikimedia("Francois-Pier", "Francois-Pier"));
        get_row(gui, ctx, ui, "Ll");
        ui.add(wikimedia("NikNaks", "NikNaks"));
        get_row(gui, ctx, ui, "Hh");
        ui.add(link("greenchess.net", "https://greenchess.net".to_string()));
        get_row(gui, ctx, ui, "IiMmOoWw");
        ui.heading("\nCC-BY-SA 4.0");
        ui.add(wikimedia("Sunny3113", "Sunny3113"));
        get_row(gui, ctx, ui, "ZzXxU");
        ui.add(wikimedia("Iago Casabiell González", "Iagocasabiell"));
        get_row(gui, ctx, ui, "Ee");
        ui.heading("\nCC0");
        ui.add(wikipedia("CheChe"));
        ui.add(get_icon(gui, ctx, 'u'));
      });
    }
    #[cfg(feature = "sound")]
    Credits::Sound => {
      ui.heading("The sound effects for piece moving were done by:");
      ui.add(github("Enigmahack"));
      ui.heading("They are licensed under AGPLv3+");
    }
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
    [ICON_SIZE as f32, ICON_SIZE as f32],
  )
}

fn github(name: &str) -> Hyperlink {
  link(name, "https://github.com/".to_string() + name)
}

fn wikipedia(name: &str) -> Hyperlink {
  link(
    name.to_string() + ":",
    "https://en.wikipedia.org/wiki/User:".to_string() + name,
  )
}

fn wikimedia(name: &str, username: &str) -> Hyperlink {
  link(
    name.to_string() + ":",
    "https://commons.wikimedia.org/wiki/User:".to_string() + username,
  )
}

fn link(name: impl Into<String>, link: String) -> Hyperlink {
  Hyperlink::from_label_and_url(size(name, TEXT_SIZE), link)
}

fn size(text: impl Into<String>, size: f32) -> RichText {
  RichText::new(text).size(size)
}

fn main() {
  let size = ICON_SIZE as u32;
  let mut pixmap = Pixmap::new(size, size).unwrap();
  resvg::render(
    &images::get()[11],
    usvg::FitTo::Size(size, size),
    tiny_skia::Transform::default(),
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
    ..Default::default()
  };

  eframe::run_native(
    "Liberty Chess",
    options,
    Box::new(|cc| Box::new(LibertyChessGUI::new(&cc.egui_ctx))),
  );
}
