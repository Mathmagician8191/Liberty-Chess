use crate::helpers::{populate_dropdown, raw_text_edit, NumericalInput};
use crate::themes::Colours;
use crate::LibertyChessGUI;
use core::time::Duration;
use eframe::egui::{ComboBox, Context, RichText, TopBottomPanel, Ui};
use liberty_chess::clock::{Clock, Type};

const MAX_TIME: u64 = 360;

pub fn draw(ctx: &Context, clock: &mut Clock, flipped: bool) {
  clock.update();
  let (mut white, mut black) = clock.get_clocks();
  if flipped {
    (black, white) = (white, black);
  }
  let mut white_text = RichText::new(print_clock(white));
  let mut black_text = RichText::new(print_clock(black));
  let color = if clock.is_flagged() {
    Colours::Check
  } else {
    Colours::Selected
  };
  if clock.to_move() ^ flipped {
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
  #[cfg(not(feature = "benchmarking"))]
  ctx.request_repaint_after(Duration::from_millis(100));
}

pub(crate) fn draw_edit(gui: &mut LibertyChessGUI, ui: &mut Ui, size: f32) {
  ComboBox::from_id_source("Clock")
    .selected_text("Clock: ".to_owned() + &gui.clock_type.to_string())
    .show_ui(ui, |ui| {
      populate_dropdown(ui, &mut gui.clock_type);
    });
  match gui.clock_type {
    Type::None => (),
    Type::Increment => {
      ui.horizontal_top(|ui| {
        ui.label("Time (min):".to_owned());
        let value = input(ui, size, gui.clock_data[0]);
        gui.clock_data[0] = value;
        gui.clock_data[1] = value;
        ui.label("Increment (s):");
        let value = input(ui, size, gui.clock_data[2]);
        gui.clock_data[2] = value;
        gui.clock_data[3] = value;
      });
    }
    Type::Handicap => {
      ui.horizontal_top(|ui| {
        ui.label("White Time (min):");
        gui.clock_data[0] = input(ui, size, gui.clock_data[0]);
        ui.label("Increment (s):");
        gui.clock_data[2] = input(ui, size, gui.clock_data[2]);
      });
      ui.horizontal_top(|ui| {
        ui.label("Black Time (min):");
        gui.clock_data[1] = input(ui, size, gui.clock_data[1]);
        ui.label("Increment (s):");
        gui.clock_data[3] = input(ui, size, gui.clock_data[3]);
      });
    }
  }
}

pub fn input(ui: &mut Ui, size: f32, input: u64) -> u64 {
  let mut input = NumericalInput::<u64>::new(input, 0, MAX_TIME);
  raw_text_edit(ui, size, &mut input);
  input.get_value()
}

fn print_clock(time: Duration) -> String {
  let secs = time.as_secs();
  if secs >= 60 {
    // Minutes and seconds
    (secs / 60).to_string() + &format!(":{:0>2}", secs % 60)
  } else {
    let millis = time.as_millis();
    secs.to_string() + &format!(".{}", (millis / 100) % 10)
  }
}
