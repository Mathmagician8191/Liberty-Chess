use crate::{switch_screen, LibertyChessGUI, Screen};
use core::str::FromStr;
use eframe::egui;
use eframe::epaint::Vec2;
use egui::color_picker::{color_edit_button_srgba, Alpha};
use egui::load::SizedTexture;
use egui::{Color32, Context, Image, TextBuffer, TextEdit, Ui};
use enum_iterator::{all, Sequence};
use liberty_chess::parsing::to_piece;

#[cfg(feature = "sound")]
use liberty_chess::{Board, Gamestate};
#[cfg(feature = "sound")]
use sound::{Effect, Engine};

//sizes of things
pub const ICON_SIZE: u32 = 48;
#[allow(clippy::cast_precision_loss)]
const ICON_SIZE_FLOAT: f32 = ICON_SIZE as f32;

pub(crate) fn menu_button(gui: &mut LibertyChessGUI, ui: &mut Ui) {
  if ui.button("Menu").clicked() {
    switch_screen(gui, Screen::Menu);
  }
}

pub(crate) fn get_icon<'a>(gui: &mut LibertyChessGUI, ctx: &Context, piece: char) -> Image<'a> {
  Image::new(SizedTexture {
    id: gui.get_image(ctx, to_piece(piece).unwrap(), ICON_SIZE),
    size: Vec2 {
      x: ICON_SIZE_FLOAT,
      y: ICON_SIZE_FLOAT,
    },
  })
}

pub(crate) fn get_fen(gui: &LibertyChessGUI) -> String {
  if let Screen::Game(ref gamestate) = gui.screen {
    if gamestate.promotion_available() {
      gui
        .undo
        .last()
        .expect("Promotion available with no previous position")
    } else {
      gamestate
    }
    .to_string()
  } else {
    String::new()
  }
}

pub fn colour_edit(ui: &mut Ui, colour: &mut Color32, text: &'static str) {
  ui.horizontal(|ui| {
    color_edit_button_srgba(ui, colour, Alpha::Opaque);
    ui.label(text);
  });
}

pub fn populate_dropdown<T: Copy + PartialEq + Sequence + ToString>(ui: &mut Ui, selected: &mut T) {
  for item in all::<T>() {
    ui.selectable_value(selected, item, item.to_string());
  }
}

pub fn populate_dropdown_transform<T: Sequence + ToString, S: PartialEq>(
  ui: &mut Ui,
  selected: &mut S,
  transform: impl Fn(T) -> S,
) {
  for item in all::<T>() {
    let string = item.to_string();
    ui.selectable_value(selected, transform(item), string);
  }
}

// Checkbox wrapper with selection/deselection sounds
pub fn checkbox(
  ui: &mut Ui,
  checked: &mut bool,
  text: &'static str,
  #[cfg(feature = "sound")] player: Option<&mut Engine>,
) -> bool {
  let clicked = ui.checkbox(checked, text).clicked();
  #[cfg(feature = "sound")]
  if let Some(player) = player {
    if clicked {
      player.play(&if *checked {
        Effect::Enable
      } else {
        Effect::Disable
      });
    }
  }
  clicked
}

// Wrappers for text editing

pub fn raw_text_edit(ui: &mut Ui, size: f32, input: &mut impl TextBuffer) {
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

// Converts Option<(T, S)> to (Option<T>, S)
// Uses S::default() if the option is None
pub fn unwrap_tuple<T, S: Default>(value: Option<(T, S)>) -> (Option<T>, S) {
  if let Some((optional, required)) = value {
    (Some(optional), required)
  } else {
    (None, Default::default())
  }
}

#[derive(Clone, Eq, PartialEq)]
pub struct NumericalInput<T: Copy + Ord + ToString> {
  number: T,
  min: T,
  max: T,
  string: String,
}

impl<T: Copy + Ord + ToString> NumericalInput<T> {
  pub fn new(number: T, min: T, max: T) -> Self {
    Self {
      number,
      min,
      max,
      string: number.to_string(),
    }
  }

  pub fn get_value(&self) -> T {
    self.number.max(self.min)
  }
}

impl<T: Copy + Default + Ord + ToString + FromStr> TextBuffer for NumericalInput<T> {
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
    let number = string.parse::<T>();
    if let Ok(number) = number {
      self.number = number;
      self.string = number.to_string();
    } else {
      self.number = T::default();
      self.string = String::new();
    }
  }
}

#[cfg(feature = "sound")]
pub fn update_sound(board: &Board, capture: bool) -> Effect {
  match board.state() {
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Effect::Victory,
    Gamestate::Stalemate | Gamestate::Repetition | Gamestate::FiftyMove | Gamestate::Material => {
      Effect::Draw
    }
    Gamestate::InProgress => {
      if board.in_check() {
        Effect::Check
      } else if capture {
        Effect::Capture
      } else {
        Effect::Move
      }
    }
  }
}
