use crate::GameMode::*;
use crate::Screen::*;
use eframe::egui;
use egui::widget_text::RichText;
use egui::{Color32, ColorImage, Context, Image, Sense, TextureFilter, TextureHandle, Ui};
use enum_iterator::{all, Sequence};
use liberty_chess::{Board, Piece};
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;
use usvg::{FitTo, Options, Tree};

const BENCHMARKING: bool = false;

const MENU_TEXT: &str = "Main Menu";

#[derive(Clone, Copy, Sequence)]
enum HelpPage {
  PawnForward,
  PawnCapture,
  PawnDouble,
  Knight,
  Bishop,
  Rook,
  Queen,
  King,
  Archbishop,
  Chancellor,
  Camel,
  Zebra,
  Mann,
  Nightrider,
  Champion,
  Centaur,
  Amazon,
  Elephant,
  Obstacle,
  Wall,
}

impl HelpPage {
  fn title(self) -> &'static str {
    match self {
      HelpPage::PawnForward => "Pawns",
      HelpPage::PawnCapture => "Pawns 2",
      HelpPage::PawnDouble => "Pawns 3",
      HelpPage::Knight => "Knight",
      HelpPage::Bishop => "Bishop",
      HelpPage::Rook => "Rook",
      HelpPage::Queen => "Queen",
      HelpPage::King => "King",
      HelpPage::Archbishop => "Archbishop",
      HelpPage::Chancellor => "Chancellor",
      HelpPage::Camel => "Camel",
      HelpPage::Zebra => "Zebra",
      HelpPage::Mann => "Mann",
      HelpPage::Nightrider => "Nightrider",
      HelpPage::Champion => "Champion",
      HelpPage::Centaur => "Centaur",
      HelpPage::Amazon => "Amazon",
      HelpPage::Elephant => "Elephant",
      HelpPage::Obstacle => "Obstacle",
      HelpPage::Wall => "Wall",
    }
  }

  fn board(self) -> Board {
    match self {
      HelpPage::PawnForward => Board::new("7/7/7/7/3P3/7/7 w").unwrap(),
      HelpPage::PawnCapture => Board::new("7/7/2ppp2/3P3/7/7/7 w").unwrap(),
      HelpPage::PawnDouble => Board::new("7/7/7/7/7/3P3/7 w").unwrap(),
      HelpPage::Knight => Board::new("7/7/7/3N3/7/7/7 w").unwrap(),
      HelpPage::Bishop => Board::new("7/ppppppp/7/3B3/7/7/7 w").unwrap(),
      HelpPage::Rook => Board::new("7/ppppppp/7/3R3/7/7/7 w").unwrap(),
      HelpPage::Queen => Board::new("7/ppppppp/7/3Q3/7/7/7 w").unwrap(),
      HelpPage::King => Board::new("7/7/7/3K3/7/7/7 w").unwrap(),
      HelpPage::Archbishop => Board::new("7/ppppppp/7/3A3/7/7/7 w").unwrap(),
      HelpPage::Chancellor => Board::new("7/ppppppp/7/3C3/7/7/7 w").unwrap(),
      HelpPage::Camel => Board::new("7/7/7/3L3/7/7/7 w").unwrap(),
      HelpPage::Zebra => Board::new("7/7/7/3Z3/7/7/7 w").unwrap(),
      HelpPage::Mann => Board::new("7/7/7/3X3/7/7/7 w").unwrap(),
      HelpPage::Nightrider => Board::new("9/9/9/9/4I4/9/9/9/9 w").unwrap(),
      HelpPage::Champion => Board::new("7/7/7/3H3/7/7/7 w").unwrap(),
      HelpPage::Centaur => Board::new("7/7/7/3U3/7/7/7 w").unwrap(),
      HelpPage::Amazon => Board::new("7/ppppppp/7/3M3/7/7/7 w").unwrap(),
      HelpPage::Elephant => Board::new("7/7/7/3E3/7/7/7 w").unwrap(),
      HelpPage::Obstacle => Board::new("7/ppppppp/7/3O3/7/7/7 w").unwrap(),
      HelpPage::Wall => Board::new("7/ppppppp/7/3W3/7/7/7 w").unwrap(),
    }
  }

  fn selected(self) -> (usize, usize) {
    match self {
      HelpPage::PawnForward => (2, 3),
      HelpPage::PawnCapture => (3, 3),
      HelpPage::PawnDouble => (1, 3),
      HelpPage::Knight => (3, 3),
      HelpPage::Bishop => (3, 3),
      HelpPage::Rook => (3, 3),
      HelpPage::Queen => (3, 3),
      HelpPage::King => (3, 3),
      HelpPage::Archbishop => (3, 3),
      HelpPage::Chancellor => (3, 3),
      HelpPage::Camel => (3, 3),
      HelpPage::Zebra => (3, 3),
      HelpPage::Mann => (3, 3),
      HelpPage::Nightrider => (4, 4),
      HelpPage::Champion => (3, 3),
      HelpPage::Centaur => (3, 3),
      HelpPage::Amazon => (3, 3),
      HelpPage::Elephant => (3, 3),
      HelpPage::Obstacle => (3, 3),
      HelpPage::Wall => (3, 3),
    }
  }

  fn description(self) -> &'static str {
    match self {
      HelpPage::PawnForward => "The pawn moves one square forward.",
      HelpPage::PawnCapture => "The pawn cannot capture forwards, but can move diagonally to capture.",
      HelpPage::PawnDouble => "The pawn can move multiple squares on its first move. The number of squares depends on the gamemode.",
      HelpPage::Knight => "The Knight jumps a set number of squares in each direction, including over other pieces.",
      HelpPage::Bishop => "The Bishop moves diagonally, but cannot go past another piece. The Bishop is confined to squares of the same colour it started on.",
      HelpPage::Rook => "The Rook moves horizontally or vertically, but cannot go past another piece.",
      HelpPage::Queen => "The Queen moves as the combination of the Bishop and the Rook.",
      HelpPage::King => "The King moves one square in any direction, and has a special move called castling (covered later). Putting the King in a position where it cannot escape is the object of the game.",
      HelpPage::Archbishop => "The Archbishop moves as the combination of the Bishop and the Knight.",
      HelpPage::Chancellor => "The Archbishop moves as the combination of the Rook and the Knight.",
      HelpPage::Camel => "The Camel moves like the Knight, only a different number of squares to it. The Camel is confined to squares of a the same colour it started on.",
      HelpPage::Zebra => "The Zebra moves like the Knight, only a different number of squares to it.",
      HelpPage::Mann => "The Mann moves one square in any direction.",
      HelpPage::Nightrider => "The Nightrider can make multiple knight jumps at once in the same direction, but cannot go past another piece on one of the knight-jump destination squares.",
      HelpPage::Champion => "The Champion can go 1 or 2 spaces in any direction, and can jump over other pieces. However, it cannot make a Knight move.",
      HelpPage::Centaur => "The Centaur moves as the combination of the Knight and the Mann.",
      HelpPage::Amazon => "The Amazon moves as a combination of the Queen and the Knight.",
      HelpPage::Elephant => "The Elephant moves like a Mann, but is immune to capture from pieces other than another Elephant or a King.",
      HelpPage::Obstacle => "The Obstacle can teleport to any empty square on the board, but cannot capture other pieces.",
      HelpPage::Wall => "The Wall moves like the Obstacle, but it can only be captured by an Elephant or King",
    }
  }
}

enum Colours {
  BlackSquare,
  WhiteSquare,
  Moved,
  Selected,
  ValidMoveBlack,
  ValidMoveWhite,
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
      Colours::ValidMoveBlack => Color32::from_rgb(120, 144, 108),
      Colours::ValidMoveWhite => Color32::from_rgb(180, 225, 168),
      Colours::Threatened => Color32::from_rgb(200, 128, 0),
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

enum Screen {
  MainMenu,
  Game,
  Help,
  Credits,
}

struct LibertyChessGUI {
  // current screen
  screen: Screen,

  // fields for main menu
  fen: String,
  gamemode: GameMode,
  message: Option<String>,

  // fields for game screen
  gamestate: Option<Board>,
  selected: Option<(usize, usize)>,
  moved: Option<[(usize, usize); 2]>,
  undo: Vec<Board>,

  //field for help screen
  help_page: HelpPage,

  // images and a render cache - used on game screen
  images: [Tree; 36],
  renders: [Option<TextureHandle>; 37],

  // for measuring FPS
  instant: Instant,
  frames: u32,
  seconds: u64,
}

impl Default for LibertyChessGUI {
  fn default() -> Self {
    Self {
      screen: MainMenu,

      gamemode: Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      message: None,

      gamestate: None,
      selected: None,
      moved: None,
      undo: Vec::new(),

      help_page: HelpPage::PawnForward,

      images: get_images(),
      renders: [(); 37].map(|_| None),

      instant: Instant::now(),
      frames: 0,
      seconds: 0,
    }
  }
}

impl LibertyChessGUI {
  fn get_image(&mut self, ctx: &Context, piece: Piece, size: usize) -> TextureHandle {
    let index = match piece {
      _ if piece > 0 => (piece - 1) as usize,
      _ if piece < 0 => (17 - piece) as usize,
      _ => {
        if let Some(map) = &self.renders[36] {
          return map.clone();
        }
        let texture = ctx.load_texture(
          "square",
          ColorImage::new([size, size], Color32::from_black_alpha(0)),
          TextureFilter::Linear,
        );
        self.renders[36] = Some(texture.clone());
        return texture;
      }
    };
    if let Some(map) = &self.renders[index] {
      if map.size() == [size, size] {
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
    let texture = ctx.load_texture("piece", image, TextureFilter::Linear);
    self.renders[index] = Some(texture.clone());
    return texture;
  }
}

impl eframe::App for LibertyChessGUI {
  fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("Topbar")
      .resizable(false)
      .show(ctx, |ui| {
        egui::widgets::global_dark_light_mode_buttons(ui);
      });

    match self.screen {
      Game => {
        egui::SidePanel::right("Sidebar")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(text(MENU_TEXT)).clicked() {
              switch_screen(self, MainMenu);
            }
            if self.undo.len() > 0 && ui.button(text("Undo")).clicked() {
              self.gamestate = self.undo.pop();
              self.moved = None;
            }
          });
      }
      Help => {
        egui::SidePanel::left("Leftbar")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(text(MENU_TEXT)).clicked() {
              switch_screen(self, MainMenu);
            }
            for page in all::<HelpPage>() {
              if ui.button(text(page.title())).clicked() {
                self.help_page = page;
              }
            }
          });
        egui::TopBottomPanel::bottom("Description")
          .resizable(false)
          .show(ctx, |ui| ui.heading(text(self.help_page.description())));
      }
      Credits | MainMenu => (),
    };

    egui::CentralPanel::default().show(ctx, |ui| {
      match self.screen {
        MainMenu => draw_menu(self, ctx, ui),
        Game => draw_game(self, ctx),
        Help => draw_help(self, ctx),
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
    // Add no delay between rendering frames when benchmarking
    if BENCHMARKING {
      ctx.request_repaint_after(Duration::ZERO);
    } else {
      ctx.request_repaint_after(Duration::from_millis(200));
    }
  }
}

fn switch_screen(gui: &mut LibertyChessGUI, screen: Screen) {
  match gui.screen {
    MainMenu => gui.message = None,
    Game => {
      gui.selected = None;
      gui.moved = None;
      gui.undo = Vec::new();
    }
    Help => gui.selected = None,
    Credits => (),
  }
  gui.screen = screen;
}

fn render_board(
  gui: &mut LibertyChessGUI,
  ctx: &Context,
  ui: &mut Ui,
  gamestate: Board,
  clickable: bool,
) {
  let available_size = ctx.available_rect().size();
  let rows = gamestate.height;
  let columns = gamestate.width;
  let row_size = (available_size.y / (rows as f32)).floor();
  let column_size = (available_size.x / (columns as f32)).floor();
  let size = f32::min(row_size, column_size);
  egui::Grid::new("Board")
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
          if let Some(start) = gui.selected {
            if gamestate.check_pseudolegal(start, coords) {
              colour = if gamestate.pieces[coords] == 0 {
                if (i + j) % 2 == 0 {
                  Colours::ValidMoveBlack
                } else {
                  Colours::ValidMoveWhite
                }
              } else {
                Colours::Threatened
              }
            }
          }
          if Some(coords) == gui.selected {
            colour = Colours::Selected;
          }
          let texture = gui.get_image(ctx, piece, size as usize);
          let icon = Image::new(texture.id(), [size, size]).bg_fill(colour.value());
          let response = ui.add(icon).interact(Sense::click());
          if clickable && response.clicked() {
            if let Some(selected) = gui.selected {
              if let Some(gamestate) = &mut gui.gamestate {
                if gamestate.check_pseudolegal(selected, coords) {
                  gui.undo.push(gamestate.clone());
                  gamestate.make_move(selected, coords);
                  gui.moved = Some([selected, coords]);
                }
              }
              gui.selected = None;
            } else {
              let piece = gamestate.pieces[coords];
              if piece != 0 && gamestate.to_move == (piece > 0) {
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
    if ui.button(text("Help")).clicked() {
      switch_screen(gui, Help);
    }
    if ui.button(text("Credits")).clicked() {
      switch_screen(gui, Credits);
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
        switch_screen(gui, Game);
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

fn draw_game(gui: &mut LibertyChessGUI, ctx: &Context) {
  if let Some(gamestate) = gui.gamestate.clone() {
    egui::Area::new("Board")
      .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
      .show(ctx, |ui| render_board(gui, ctx, ui, gamestate, true));
  } else {
    println!("On Game screen with no gamestate");
    switch_screen(gui, MainMenu);
  }
}

fn draw_help(gui: &mut LibertyChessGUI, ctx: &Context) {
  gui.selected = Some(gui.help_page.selected());
  egui::Area::new("Board")
    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    .show(ctx, |ui| {
      render_board(gui, ctx, ui, gui.help_page.board(), false)
    });
}

fn draw_credits(gui: &mut LibertyChessGUI, ui: &mut Ui) {
  ui.heading(text("Credits - TODO"));
  if ui.button(text(MENU_TEXT)).clicked() {
    switch_screen(gui, MainMenu);
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
  let mut options = eframe::NativeOptions::default();
  // Disable vsync when benchmarking to remove the framerate limit
  options.vsync = !BENCHMARKING;
  eframe::run_native(
    "Liberty Chess",
    options,
    Box::new(|_cc| Box::new(LibertyChessGUI::default())),
  )
}
