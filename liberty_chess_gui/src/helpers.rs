use crate::{switch_screen, LibertyChessGUI, Screen};
use eframe::egui::{Context, Image, TextBuffer, TextEdit, Ui};

//sizes of things
pub const ICON_SIZE: u32 = 48;
#[allow(clippy::cast_precision_loss)]
const ICON_SIZE_FLOAT: f32 = ICON_SIZE as f32;

#[cfg(feature = "clock")]
const MAX_TIME: u64 = 360;

pub(crate) fn menu_button(gui: &mut LibertyChessGUI, ui: &mut Ui) {
  if ui.button("Menu").clicked() {
    switch_screen(gui, Screen::Menu);
  }
}

pub fn text_edit(ui: &mut Ui, char_size: f32, min_size: f32, string: &mut String) {
  let space = f32::min(
    ui.available_size().x,
    f32::max(char_size * string.len() as f32, min_size),
  );
  ui.add_sized([space, 0.0], TextEdit::singleline(string));
}

pub(crate) fn get_icon(gui: &mut LibertyChessGUI, ctx: &Context, piece: char) -> Image {
  Image::new(
    gui.get_image(ctx, liberty_chess::to_piece(piece).unwrap(), ICON_SIZE),
    [ICON_SIZE_FLOAT, ICON_SIZE_FLOAT],
  )
}

pub(crate) fn get_fen(gui: &LibertyChessGUI) -> String {
  if let Screen::Game(ref gamestate) = gui.screen {
    if gamestate.promotion_available() {
      gui
        .undo
        .last()
        .expect("Promotion available with no previous position")
        .to_string()
    } else {
      gamestate.to_string()
    }
  } else {
    String::new()
  }
}

#[cfg(feature = "clock")]
pub fn clock_input(ui: &mut Ui, size: f32, input: u64) -> u64 {
  let mut input = NumericalInput::new(input);
  ui.add_sized([size, 0.0], TextEdit::singleline(&mut input));
  input.get_value()
}

#[cfg(feature = "clock")]
struct NumericalInput {
  number: u64,
  string: String,
}

impl NumericalInput {
  fn new(number: u64) -> Self {
    Self {
      number,
      string: number.to_string(),
    }
  }

  fn get_value(&self) -> u64 {
    self.number
  }
}

impl TextBuffer for NumericalInput {
  fn is_mutable(&self) -> bool {
    true
  }

  fn as_str(&self) -> &str {
    &self.string
  }

  fn insert_text(&mut self, text: &str, index: usize) -> usize {
    let mut string = self.string.clone();
    let chars = string.insert_text(text, index);
    match string.parse::<u64>() {
      Ok(mut number) => {
        number = u64::min(number, MAX_TIME);
        self.number = number;
        self.string = number.to_string();
        chars
      }
      Err(_) => 0,
    }
  }

  fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
    let mut string = self.string.clone();
    string.delete_char_range(char_range);
    let number = string.parse::<u64>().unwrap_or(0);
    self.number = number;
    self.string = number.to_string();
  }
}
