use crate::colours::Colours;
use crate::credits::Credits;
use crate::gamemodes::{GameMode, Presets};
use crate::help_page::HelpPage;
use crate::themes::Theme;
use eframe::egui;
use egui::{
  Color32, ColorImage, ComboBox, Context, FontFamily, FontId, Image, RichText, TextStyle,
  TextureFilter, TextureHandle, Ui,
};
use enum_iterator::all;
use liberty_chess::{print_secs, Board, Clock, ClockType, Piece};
use std::time::{Duration, Instant};

// enums in own file
mod colours;
mod credits;
mod gamemodes;
mod help_page;
mod themes;

// file to load images
mod images;

const BENCHMARKING: bool = false;

const MENU_TEXT: &str = "Back to Menu";

const ICON_SIZE: usize = 50;

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

  // fields for main menu
  fen: String,
  gamemode: GameMode,
  message: Option<String>,
  clock_type: ClockType,
  clock_data: [u64; 4],

  // fields for game screen
  gamestate: Option<Board>,
  selected: Option<(usize, usize)>,
  moved: Option<[(usize, usize); 2]>,
  undo: Vec<Board>,
  clock: Option<Clock>,

  //field for help screen
  help_page: HelpPage,

  // field for credits
  credits: Credits,

  // images and a render cache - used on game screen
  images: [usvg::Tree; 36],
  renders: [Option<TextureHandle>; 37],

  // for measuring FPS
  instant: Instant,
  frames: u32,
  seconds: u64,
}

impl Default for LibertyChessGUI {
  fn default() -> Self {
    Self {
      screen: Screen::Menu,

      theme: Theme::Dark,

      gamemode: GameMode::Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      message: None,
      clock_type: ClockType::None,
      clock_data: [10, 10, 10, 10],

      gamestate: None,
      selected: None,
      moved: None,
      undo: Vec::new(),
      clock: None,

      help_page: HelpPage::PawnForward,
      credits: Credits::Coding,

      images: images::get(),
      renders: [(); 37].map(|_| None),

      instant: Instant::now(),
      frames: 0,
      seconds: 0,
    }
  }
}

impl LibertyChessGUI {
  fn new(ctx: &Context) -> Self {
    let mut style = (*ctx.style()).clone();
    let font = FontId::new(24.0, FontFamily::Proportional);
    style.text_styles = [
      (TextStyle::Heading, font.clone()),
      (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
      (TextStyle::Button, font),
    ]
    .into();
    ctx.set_style(style);
    Self::default()
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
    let mut pixmap = tiny_skia::Pixmap::new(size as u32, size as u32).unwrap();
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
        ComboBox::from_id_source("Theme")
          .selected_text(size("Theme: ".to_string() + &self.theme.to_string(), 16.0))
          .show_ui(ui, |ui| {
            for theme in all::<Theme>() {
              ui.selectable_value(&mut self.theme, theme, size(theme.to_string(), 16.0));
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
          });
        if let Some(clock) = &mut self.clock {
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
            .show(ctx, |ui| {
              ui.heading(white_text);
            });
          egui::TopBottomPanel::top("Black Clock")
            .resizable(false)
            .show(ctx, |ui| {
              ui.heading(black_text);
            });
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
    if BENCHMARKING {
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
          if let Some(start) = gui.selected {
            if gamestate.check_pseudolegal(start, coords) {
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
                  gui.undo.push(gamestate.clone());
                  gamestate.make_move(selected, coords);
                  gui.moved = Some([selected, coords]);
                  if let Some(clock) = &mut gui.clock {
                    clock.switch_clocks();
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
        gui.gamestate = Some(board.clone());
        match gui.clock_type {
          ClockType::None => gui.clock = None,
          ClockType::Increment | ClockType::Handicap => {
            gui.clock = Some(Clock::new(gui.clock_data, board.to_move()));
          }
        }
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
      for clock_type in all::<ClockType>() {
        ui.selectable_value(&mut gui.clock_type, clock_type, clock_type.to_string());
      }
    });
  match gui.clock_type {
    ClockType::None => (),
    ClockType::Increment => {
      ui.horizontal_top(|ui| {
        let mut input: Vec<String> = gui.clock_data.iter().map(|x| x.to_string()).collect();
        ui.label("Time (minutes):".to_string());
        text_edit(ui, 8.0, 25.0, &mut input[0]);
        ui.label("Increment (seconds):");
        text_edit(ui, 8.0, 25.0, &mut input[2]);
        if let Ok(value) = input[0].parse::<u64>() {
          gui.clock_data[0] = value;
          gui.clock_data[1] = value;
        }
        if let Ok(value) = input[2].parse::<u64>() {
          gui.clock_data[2] = value;
          gui.clock_data[3] = value;
        }
      });
    }
    ClockType::Handicap => {
      let mut input: Vec<String> = gui.clock_data.iter().map(|x| x.to_string()).collect();
      ui.horizontal_top(|ui| {
        ui.label("White Time (minutes):");
        text_edit(ui, 8.0, 25.0, &mut input[0]);
        ui.label("White Increment (seconds):");
        text_edit(ui, 8.0, 25.0, &mut input[2]);
      });
      ui.horizontal_top(|ui| {
        ui.label("Black Time (minutes):");
        text_edit(ui, 8.0, 25.0, &mut input[1]);
        ui.label("Black Increment (seconds):");
        text_edit(ui, 8.0, 25.0, &mut input[3]);
      });
      for (i, value) in input.iter().enumerate() {
        if let Ok(value) = value.parse::<u64>() {
          gui.clock_data[i] = value;
        }
      }
    }
  }
}

fn draw_game(gui: &mut LibertyChessGUI, ctx: &Context) {
  if let Some(gamestate) = gui.gamestate.clone() {
    let mut clickable = true;
    if let Some(clock) = &gui.clock {
      if clock.is_flagged() {
        gui.selected = None;
        clickable = false;
      }
    }
    egui::Area::new("Board")
      .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
      .show(ctx, |ui| render_board(gui, ctx, ui, &gamestate, clickable));
  } else {
    println!("On Game screen with no gamestate");
    switch_screen(gui, Screen::Menu);
  }
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
      ui.hyperlink_to("Mathmagician8191", "https://github.com/Mathmagician8191");
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
        ui.heading("Cburnett:");
        get_row(gui, ctx, ui, "AaCc");
        ui.heading("Francois-Pier:");
        get_row(gui, ctx, ui, "Ll");
        ui.heading("NikNaks:");
        get_row(gui, ctx, ui, "Hh");
        ui.hyperlink("greenchess.net");
        get_row(gui, ctx, ui, "IiMmOoWw");
        ui.heading("\nCC-BY-SA 4.0");
        ui.heading("Sunny3113:");
        get_row(gui, ctx, ui, "ZzXxU");
        ui.heading("Iago Casabiell González:");
        get_row(gui, ctx, ui, "Ee");
        ui.heading("\nCC0");
        ui.heading("CheChe:");
        ui.add(get_icon(gui, ctx, 'u'));
      });
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

fn size(text: String, size: f32) -> RichText {
  RichText::new(text).size(size)
}

fn main() {
  let options = eframe::NativeOptions {
    // Disable vsync when benchmarking to remove the framerate limit
    vsync: !BENCHMARKING,
    ..Default::default()
  };

  eframe::run_native(
    "Liberty Chess",
    options,
    Box::new(|cc| Box::new(LibertyChessGUI::new(&cc.egui_ctx))),
  );
}
