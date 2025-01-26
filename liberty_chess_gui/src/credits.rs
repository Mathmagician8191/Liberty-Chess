use crate::helpers::get_icon;
use crate::LibertyChessGUI;
use eframe::egui::{Color32, Context, Hyperlink, ScrollArea, Ui, WidgetText};
use enum_iterator::Sequence;

#[derive(Clone, Copy, Sequence)]
pub enum Credits {
  Coding,
  Images,
  #[cfg(feature = "sound")]
  Sound,
  #[cfg(feature = "music")]
  Music,
}

impl Credits {
  pub const fn title(self) -> &'static str {
    match self {
      Self::Coding => "Coding",
      Self::Images => "Images",
      #[cfg(feature = "sound")]
      Self::Sound => "Sound effects",
      #[cfg(feature = "music")]
      Self::Music => "Music",
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

// convenient wrappers for links

fn github(ui: &mut Ui, name: &str) {
  link(ui, name, format!("https://github.com/{name}"));
}

fn wikipedia(ui: &mut Ui, name: &str) {
  link(
    ui,
    format!("{name}:"),
    format!("https://en.wikipedia.org/wiki/User:{name}"),
  );
}

fn wikimedia(ui: &mut Ui, name: &str, username: &str) {
  link(
    ui,
    format!("{name}:"),
    format!("https://commons.wikimedia.org/wiki/User:{username}"),
  );
}

fn link(ui: &mut Ui, name: impl Into<WidgetText>, link: impl ToString) {
  let name: WidgetText = name.into();
  ui.add(Hyperlink::from_label_and_url(name.color(Color32::BLUE), link));
}

pub(crate) fn draw(gui: &mut LibertyChessGUI, ctx: &Context, ui: &mut Ui) {
  match gui.credits {
    Credits::Coding => {
      ui.label("Programming done by:");
      github(ui, "Mathmagician8191");
      ui.label("The code is licensed under GPL v3 and can be found here:");
      let code_link = "https://github.com/Mathmagician8191/Liberty-Chess";
      link(ui, code_link, code_link);
      ui.label(
        "Credit to the Stockfish Discord for helping me with Oxidation, the built-in engine",
      );
    }
    Credits::Images => {
      ScrollArea::vertical().show(ui, |ui| {
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
        link(ui, "greenchess.net", "https://greenchess.net");
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
      ui.label("Piece moving and victory/draw sound effects were done by:");
      github(ui, "Enigmahack");
      ui.label("They are licensed under AGPLv3+");
      ui.label("Illegal move, menu navigation and checkbox sounds done by:");
      link(
        ui,
        "Aviitone",
        "https://open.spotify.com/artist/1S4BPnkEXlh3fptVfr5JNf",
      );
      ui.label("They are licensed under CC BY-NC-SA 4.0");
    }
    #[cfg(feature = "music")]
    Credits::Music => {
      ui.label("Music composed by:");
      link(
        ui,
        "Aviitone",
        "https://open.spotify.com/artist/1S4BPnkEXlh3fptVfr5JNf",
      );
      ui.label("It is licensed under CC BY-NC-SA 4.0");
      ui.label("List of tracks:");
      ui.label("\"Hydrangeas for a Friend\"");
      ui.label("\"Renard Lullaby\"");
      ui.label("\"Cognitive Ambience\"");
      ui.label("\"Recursive Thinking\"");
      ui.label("\"Perplex Objector\"");
      ui.label("\"Wasted Opportunity\"");
      ui.label("\"Brillfish\"");
    }
  }
}
