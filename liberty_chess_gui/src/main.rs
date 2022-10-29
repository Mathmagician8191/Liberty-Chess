use crate::colours::Colours;
use crate::credits::Credits;
use crate::gamemodes::{GameMode, Presets};
use crate::help_page::HelpPage;
use crate::themes::Theme;
use crate::Screen::*;
use eframe::egui;
use egui::{
  Color32, ColorImage, ComboBox, Context, FontFamily, FontId, Image, RichText, TextStyle,
  TextureFilter, TextureHandle, Ui,
};
use enum_iterator::all;
use liberty_chess::{Board, Piece};
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
  MainMenu,
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

  // fields for game screen
  gamestate: Option<Board>,
  selected: Option<(usize, usize)>,
  moved: Option<[(usize, usize); 2]>,
  undo: Vec<Board>,

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
      screen: MainMenu,

      theme: Theme::Dark,

      gamemode: GameMode::Preset(Presets::Standard),
      fen: Presets::Standard.value(),
      message: None,

      gamestate: None,
      selected: None,
      moved: None,
      undo: Vec::new(),

      help_page: HelpPage::PawnForward,
      credits: Credits::Coding,

      images: images::get_images(),
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
      (TextStyle::Button, font.clone()),
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
    return texture.id();
  }
}

impl eframe::App for LibertyChessGUI {
  fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
    let theme = self.theme;
    egui::TopBottomPanel::top("Topbar")
      .resizable(false)
      .show(ctx, |ui| {
        ComboBox::from_label("")
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
    match self.screen {
      Game => {
        egui::SidePanel::right("Sidebar")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(MENU_TEXT).clicked() {
              switch_screen(self, MainMenu);
            }
            if self.undo.len() > 0 && ui.button("Undo").clicked() {
              self.gamestate = self.undo.pop();
              self.moved = None;
            }
          });
      }
      Help => {
        egui::SidePanel::left("Leftbar")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(MENU_TEXT).clicked() {
              switch_screen(self, MainMenu);
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
              for page in all::<HelpPage>() {
                let mut text = RichText::new(page.title());
                if page == self.help_page {
                  text = text.color(Colours::ValidBlack.value());
                }
                let button = egui::Button::new(text);
                if ui.add(button).clicked() {
                  self.help_page = page;
                }
              }
            });
          });
        egui::TopBottomPanel::bottom("Description")
          .resizable(false)
          .show(ctx, |ui| ui.heading(self.help_page.description()));
      }
      Credits => {
        egui::SidePanel::left("Leftbar")
          .resizable(false)
          .show(ctx, |ui| {
            if ui.button(MENU_TEXT).clicked() {
              switch_screen(self, MainMenu);
            }
            ui.heading("Credits:");
            for page in all::<Credits>() {
              if ui.button(page.title()).clicked() {
                self.credits = page;
              }
            }
          });
      }
      MainMenu => (),
    };

    egui::CentralPanel::default().show(ctx, |ui| {
      match self.screen {
        MainMenu => draw_menu(self, ctx, ui),
        Game => draw_game(self, ctx),
        Help => draw_help(self, ctx),
        Credits => draw_credits(self, ctx, ui),
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
    Help => {
      gui.selected = None;
      gui.moved = None;
    }
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
          let piece = gamestate.pieces[coords];
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
              colour = if gamestate.pieces[coords] == 0 {
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
    if ui.button("Help").clicked() {
      switch_screen(gui, Help);
    }
    if ui.button("Credits").clicked() {
      switch_screen(gui, Credits);
    }
  });
  egui::ComboBox::from_label("")
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
    let space = f32::min(
      ui.available_size().x,
      f32::max(11.5 * gui.fen.len() as f32, 220.0),
    );
    ui.add_sized([space, 0.0], egui::TextEdit::singleline(&mut gui.fen));
  }
  if ui.button("Start Game").clicked() {
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
    ui.heading(message);
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
  gui.moved = gui.help_page.moved();
  egui::Area::new("Board")
    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
    .show(ctx, |ui| {
      render_board(gui, ctx, ui, gui.help_page.board(), false)
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
  let mut options = eframe::NativeOptions::default();
  // Disable vsync when benchmarking to remove the framerate limit
  options.vsync = !BENCHMARKING;
  eframe::run_native(
    "Liberty Chess",
    options,
    Box::new(|cc| Box::new(LibertyChessGUI::new(&cc.egui_ctx))),
  )
}
