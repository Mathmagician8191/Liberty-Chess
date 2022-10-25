use crate::GameMode::*;
use crate::Screen::*;
use eframe::egui;
use egui::widget_text::RichText;
use egui::{Color32, ColorImage, Image, Sense, TextureFilter};
use enum_iterator::{all, Sequence};
use liberty_chess::{Board, Piece};
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;
use usvg::{FitTo, Options, Tree};

enum Screen {
  MainMenu,
  Game,
  Help,
  Credits,
}

enum Colours {
  BlackSquare,
  WhiteSquare,
  Moved,
  Selected,
  ValidMove,
  Threatened,
  Check,
}

impl Colours {
  fn value(&self) -> Color32 {
    match self {
      Colours::BlackSquare => Color32::from_rgb(160, 128, 96),
      Colours::WhiteSquare => Color32::from_rgb(240, 217, 181),
      Colours::Moved => Color32::from_rgb(64, 192, 0),
      Colours::Selected => Color32::from_rgb(192, 192, 0),
      Colours::ValidMove => Color32::from_rgb(0, 192, 192),
      Colours::Threatened => Color32::from_rgb(192, 96, 0),
      Colours::Check => Color32::from_rgb(192, 0, 0),
    }
  }
}

#[derive(PartialEq)]
enum GameMode {
  Preset(Presets),
  Custom,
}

#[derive(Clone, Copy, PartialEq, Sequence)]
enum Presets {
  Standard,
  Mini,
  CapablancaRectangle,
  CapablancaSquare,
  Mongol,
  LoadedBoard,
}

impl Presets {
  fn value(&self) -> String {
    match self {
      Presets::Standard => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
      Presets::Mini => "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1".to_string(),
      Presets::CapablancaRectangle => {
        "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1".to_string()
      }
      Presets::CapablancaSquare => {
        "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1".to_string()
      }
      Presets::Mongol => "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1".to_string(),
      Presets::LoadedBoard => {
        "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1"
          .to_string()
      }
    }
  }
}

impl ToString for Presets {
  fn to_string(&self) -> String {
    match self {
      Presets::Standard => "Standard".to_string(),
      Presets::Mini => "Mini chess".to_string(),
      Presets::CapablancaRectangle => "Capablanca's chess (10x8)".to_string(),
      Presets::CapablancaSquare => "Capablanca's chess (10x10)".to_string(),
      Presets::Mongol => "Mongol chess".to_string(),
      Presets::LoadedBoard => "Loaded board".to_string(),
    }
  }
}

impl ToString for GameMode {
  fn to_string(&self) -> String {
    match self {
      Preset(preset) => preset.to_string(),
      Custom => "Custom".to_string(),
    }
  }
}

struct LibertyChessGUI {
  screen: Screen,
  fen: String,
  gamemode: GameMode,
  gamestate: Option<Board>,
  selected: Option<(usize, usize)>,
  moved: Option<[(usize, usize); 2]>,
  message: Option<String>,
  // images and a render cache
  images: [Tree; 36],
  renders: [Option<ColorImage>; 36],
  // for measuring FPS
  instant: Instant,
  frames: u32,
  seconds: u64,
}

impl LibertyChessGUI {
  fn get_image(&mut self, piece: Piece, size: usize) -> ColorImage {
    let index = match piece {
      _ if piece > 0 => (piece - 1) as usize,
      _ if piece < 0 => (17 - piece) as usize,
      _ => return ColorImage::new([size, size], Color32::from_black_alpha(0)),
    };
    if let Some(map) = &self.renders[index] {
      if map.width() == size {
        return map.clone();
      }
    }
    let mut pixmap = Pixmap::new(size as u32, size as u32).unwrap();
    resvg::render(
      &self.images[index],
      FitTo::Size(size as u32, size as u32),
      tiny_skia::Transform::default(),
      pixmap.as_mut(),
    )
    .unwrap();
    let image = egui::ColorImage::from_rgba_unmultiplied([size, size], pixmap.data());
    self.renders[index] = Some(image.clone());
    return image;
  }
}

impl Default for LibertyChessGUI {
  fn default() -> Self {
    Self {
      screen: MainMenu,
      fen: Presets::Standard.value(),
      gamemode: Preset(Presets::Standard),
      gamestate: None,
      selected: None,
      moved: None,
      message: None,
      images: get_images(),
      renders: [(); 36].map(|_| None),
      instant: Instant::now(),
      frames: 0,
      seconds: 0,
    }
  }
}

impl eframe::App for LibertyChessGUI {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      match self.screen {
        MainMenu => draw_menu(self, ctx, ui),
        Game => draw_game(self, ctx, ui),
        Help => draw_help(self, ui),
        Credits => draw_credits(self, ui),
      };
    });
    self.frames += 1;
    let duration = self.instant.elapsed().as_secs();
    if duration - self.seconds > 0 {
      self.seconds = duration;
      println!("{} FPS", self.frames);
      self.frames = 0;
    }
    ctx.request_repaint_after(Duration::from_millis(200));
    // Add no delay between rendering frames, for benchmarking
    // ctx.request_repaint_after(Duration::ZERO);
  }
}

fn draw_menu(gui: &mut LibertyChessGUI, _ctx: &egui::Context, ui: &mut egui::Ui) {
  ui.horizontal_top(|ui| {
    if ui.button(text("Help")).clicked() {
      gui.message = None;
      gui.screen = Help;
    }
    if ui.button(text("Credits")).clicked() {
      gui.message = None;
      gui.screen = Credits;
    }
  });
  egui::ComboBox::from_label(text("Select game mode"))
    .selected_text(text(&gui.gamemode.to_string()))
    .show_ui(ui, |ui| {
      for gamemode in all::<Presets>() {
        ui.selectable_value(&mut gui.gamemode, Preset(gamemode), gamemode.to_string());
      }
      ui.selectable_value(&mut gui.gamemode, Custom, "Custom")
    });
  if let Preset(preset) = gui.gamemode {
    gui.fen = preset.value();
  } else {
    let space = f32::min(
      ui.available_size().x,
      f32::max(8.0 * gui.fen.len() as f32, 100.0),
    );
    ui.add_sized([space, 0.0], egui::TextEdit::singleline(&mut gui.fen));
  }
  if ui.button(text("Start Game")).clicked() {
    match Board::new(&gui.fen) {
      Ok(board) => {
        gui.gamestate = Some(board);
        gui.message = None;
        gui.screen = Game;
      }
      Err(error) => {
        gui.message = Some(error.to_string());
      }
    }
  }
  if let Some(message) = &gui.message {
    ui.heading(text(message));
  }
}

fn draw_game(gui: &mut LibertyChessGUI, ctx: &egui::Context, ui: &mut egui::Ui) {
  if ui.button(text("Main menu")).clicked() {
    gui.screen = MainMenu;
    gui.moved = None;
  }
  if let Some(gamestate) = gui.gamestate.clone() {
    let available_size = ui.available_size();
    let rows = gamestate.height;
    let columns = gamestate.width;
    let row_size = (available_size.y / (rows as f32)).floor();
    let column_size = (available_size.x / (columns as f32)).floor();
    let size = f32::min(row_size, column_size);
    egui::Grid::new("board")
      .num_columns(columns)
      .spacing([0.0, 0.0])
      .min_col_width(size)
      .min_row_height(size)
      .show(ui, |ui| {
        for i in (0..rows).rev() {
          for j in 0..columns {
            let coords = (i, j);
            let piece = gamestate.pieces[coords];
            let mut colour = if (i + j) % 2 == 0 {
              Colours::BlackSquare
            } else {
              Colours::WhiteSquare
            };
            if let Some([from, to]) = gui.moved {
              if coords == from || coords == to {
                colour = Colours::Moved;
              }
            }
            if Some(coords) == gui.selected {
              colour = Colours::Selected;
            }
            let image = gui.get_image(piece, size as usize);
            let texture = ctx.load_texture("piece", image, TextureFilter::Linear);
            let icon = Image::new(texture.id(), [size, size]).bg_fill(colour.value());
            let response = ui.add(icon).interact(Sense::click());
            if response.clicked() {
              if let Some(selected) = gui.selected {
                if let Some(gamestate) = &mut gui.gamestate {
                  gamestate.make_move(selected, coords);
                  gui.moved = Some([selected, coords]);
                }
                gui.selected = None;
              } else {
                gui.selected = Some(coords);
              }
              println!("{}{}", j, i);
            }
          }
          ui.end_row();
        }
      });
  } else {
    println!("On Game screen with no gamestate");
    gui.screen = MainMenu;
  }
}

fn draw_help(gui: &mut LibertyChessGUI, ui: &mut egui::Ui) {
  ui.heading(text("Help - TODO"));
  if ui.button(text("Main menu")).clicked() {
    gui.screen = MainMenu;
  }
}

fn draw_credits(gui: &mut LibertyChessGUI, ui: &mut egui::Ui) {
  ui.heading(text("Credits - TODO"));
  if ui.button(text("Main menu")).clicked() {
    gui.screen = MainMenu;
  }
}

fn load_image(data: &[u8]) -> Tree {
  Tree::from_data(data, &Options::default().to_ref()).unwrap()
}

fn get_images() -> [Tree; 36] {
  [
    load_image(include_bytes!("../../resources/WPawn.svg")),
    load_image(include_bytes!("../../resources/WKnight.svg")),
    load_image(include_bytes!("../../resources/WBishop.svg")),
    load_image(include_bytes!("../../resources/WRook.svg")),
    load_image(include_bytes!("../../resources/WQueen.svg")),
    load_image(include_bytes!("../../resources/WKing.svg")),
    load_image(include_bytes!("../../resources/WArchbishop.svg")),
    load_image(include_bytes!("../../resources/WChancellor.svg")),
    load_image(include_bytes!("../../resources/WCamel.svg")),
    load_image(include_bytes!("../../resources/WZebra.svg")),
    load_image(include_bytes!("../../resources/WMann.svg")),
    load_image(include_bytes!("../../resources/WNightrider.svg")),
    load_image(include_bytes!("../../resources/WChampion.svg")),
    load_image(include_bytes!("../../resources/WCentaur.svg")),
    load_image(include_bytes!("../../resources/WAmazon.svg")),
    load_image(include_bytes!("../../resources/WElephant.svg")),
    load_image(include_bytes!("../../resources/WObstacle.svg")),
    load_image(include_bytes!("../../resources/WWall.svg")),
    load_image(include_bytes!("../../resources/BPawn.svg")),
    load_image(include_bytes!("../../resources/BKnight.svg")),
    load_image(include_bytes!("../../resources/BBishop.svg")),
    load_image(include_bytes!("../../resources/BRook.svg")),
    load_image(include_bytes!("../../resources/BQueen.svg")),
    load_image(include_bytes!("../../resources/BKing.svg")),
    load_image(include_bytes!("../../resources/BArchbishop.svg")),
    load_image(include_bytes!("../../resources/BChancellor.svg")),
    load_image(include_bytes!("../../resources/BCamel.svg")),
    load_image(include_bytes!("../../resources/BZebra.svg")),
    load_image(include_bytes!("../../resources/BMann.svg")),
    load_image(include_bytes!("../../resources/BNightrider.svg")),
    load_image(include_bytes!("../../resources/BChampion.svg")),
    load_image(include_bytes!("../../resources/BCentaur.svg")),
    load_image(include_bytes!("../../resources/BAmazon.svg")),
    load_image(include_bytes!("../../resources/BElephant.svg")),
    load_image(include_bytes!("../../resources/BObstacle.svg")),
    load_image(include_bytes!("../../resources/BWall.svg")),
  ]
}

fn text(text: &str) -> RichText {
  RichText::new(text).size(24.0)
}

fn main() {
  let options = eframe::NativeOptions::default();
  // Disable vsync for benchmarking
  // options.vsync = false;
  eframe::run_native(
    "Liberty Chess",
    options,
    Box::new(|_cc| Box::new(LibertyChessGUI::default())),
  )
}
