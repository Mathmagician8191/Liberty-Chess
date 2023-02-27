use crate::{switch_screen, LibertyChessGUI, Screen};
use core::str::FromStr;
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

fn raw_text_edit(ui: &mut Ui, size: f32, input: &mut impl TextBuffer) {
  ui.add_sized([size, 0.0], TextEdit::singleline(input));
}

pub fn char_text_edit(ui: &mut Ui, size: f32, string: &mut String) {
  let space = f32::min(
    ui.available_size().x,
    f32::max(size * 0.74 * string.len() as f32, size * 11.0),
  );
  raw_text_edit(ui, space, string);
}

pub fn label_text_edit(ui: &mut Ui, size: f32, input: &mut impl TextBuffer, label: &str) {
  ui.horizontal_top(|ui| {
    ui.label(label);
    raw_text_edit(ui, size, input);
  });
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
  let mut input = NumericalInput::<u64>::new(input, 0, MAX_TIME);
  raw_text_edit(ui, size, &mut input);
  input.get_value()
}

#[cfg(feature = "clock")]
#[derive(Eq, PartialEq)]
pub struct NumericalInput<T: Copy + ToString> {
  number: T,
  min: T,
  max: T,
  string: String,
}

impl<T: Copy + ToString> NumericalInput<T> {
  pub fn new(number: T, min: T, max: T) -> Self {
    Self {
      number,
      min,
      max,
      string: number.to_string(),
    }
  }

  pub fn get_value(&self) -> T {
    self.number
  }
}

impl<T: Copy + Ord + ToString + FromStr> TextBuffer for NumericalInput<T> {
  fn is_mutable(&self) -> bool {
    true
  }

  fn as_str(&self) -> &str {
    &self.string
  }

  fn insert_text(&mut self, text: &str, index: usize) -> usize {
    let mut string = self.string.clone();
    let chars = string.insert_text(text, index);
    match string.parse::<T>() {
      Ok(mut number) => {
        number = T::min(number, self.max);
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
    let number = T::max(string.parse::<T>().unwrap_or(self.min), self.min);
    self.number = number;
    self.string = number.to_string();
  }
}
