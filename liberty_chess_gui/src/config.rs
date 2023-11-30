use crate::themes::{GetVisuals, PresetTheme, Theme};
use core::str::FromStr;
use eframe::{egui, Storage};
use egui::{Context, FontId, TextStyle};

pub const BOARD_KEY: &str = "Board";
#[cfg(feature = "sound")]
pub const SOUND_KEY: &str = "Sound";
#[cfg(feature = "sound")]
pub const EFFECT_VOLUME_KEY: &str = "Volume";
#[cfg(feature = "music")]
pub const DRAMATIC_ENABLED_KEY: &str = "Dramatic";
#[cfg(feature = "music")]
pub const MUSIC_VOLUME_KEY: &str = "Music_Volume";

/// A Configuration parameter value.
/// Will only be saved when Modified, to allow changing the default value for users who haven't specified it.
enum Value<T> {
  Modified(T),
  Default,
}

trait Parameter<T> {
  fn default_value() -> T;
}

fn load<T: Parameter<T> + FromStr>(input: Option<String>) -> Value<T> {
  deserialise(input).map_or(Value::Default, |value| Value::Modified(value))
}

fn deserialise<T: Parameter<T> + FromStr>(input: Option<String>) -> Option<T> {
  input?.parse::<T>().ok()
}

fn save<T: Parameter<T> + ToString>(storage: &mut dyn Storage, key: &str, value: &Value<T>) {
  if let Value::Modified(value) = value {
    storage.set_string(key, value.to_string());
  }
}

fn get_value<T: Parameter<T> + Clone>(raw: &Value<T>) -> T {
  match raw {
    Value::Modified(value) => value.clone(),
    Value::Default => T::default_value(),
  }
}

const NUMBER_KEY: &str = "Numbers";
const TEXT_SIZE_KEY: &str = "Text_Size";
const THEME_KEY: &str = "Theme";
const AUTOFLIP_KEY: &str = "Autoflip";
const OPPONENTFLIP_KEY: &str = "Opponentflip";
const ADVANCED_KEY: &str = "Advanced_Settings";
const EVAL_BAR_KEY: &str = "Eval_Bar";

pub struct Configuration {
  theme: Value<Theme>,
  text_size: Value<TextSize>,
  numbers: Value<bool>,
  auto_flip: Value<bool>,
  opponent_flip: Value<bool>,
  advanced_settings: Value<bool>,
  eval_bar: Value<bool>,
}

impl Configuration {
  pub fn new(ctx: &eframe::CreationContext) -> Self {
    let config = ctx.storage.as_ref().map_or(
      Self {
        theme: Value::Default,
        text_size: Value::Default,
        numbers: Value::Default,
        auto_flip: Value::Default,
        opponent_flip: Value::Default,
        advanced_settings: Value::Default,
        eval_bar: Value::Default,
      },
      |storage| Self {
        theme: load(storage.get_string(THEME_KEY)),
        text_size: load(storage.get_string(TEXT_SIZE_KEY)),
        numbers: load(storage.get_string(NUMBER_KEY)),
        auto_flip: load(storage.get_string(AUTOFLIP_KEY)),
        opponent_flip: load(storage.get_string(OPPONENTFLIP_KEY)),
        advanced_settings: load(storage.get_string(ADVANCED_KEY)),
        eval_bar: load(storage.get_string(EVAL_BAR_KEY)),
      },
    );
    config.set_style(&ctx.egui_ctx);
    config.apply_theme(&ctx.egui_ctx);

    config
  }

  pub fn save(&self, storage: &mut dyn Storage) {
    save(storage, THEME_KEY, &self.theme);
    save(storage, TEXT_SIZE_KEY, &self.text_size);
    save(storage, NUMBER_KEY, &self.numbers);
    save(storage, AUTOFLIP_KEY, &self.auto_flip);
    save(storage, OPPONENTFLIP_KEY, &self.opponent_flip);
    save(storage, ADVANCED_KEY, &self.advanced_settings);
    save(storage, EVAL_BAR_KEY, &self.eval_bar);
  }

  // Reset every parameter to their default value
  // Currently non-functional due to https://github.com/emilk/egui/issues/2641
  // pub fn reset_all(&mut self, ctx: &Context) {
  //   self.theme = Value::Default;
  //   self.text_size = Value::Default;
  //   self.set_style(ctx);
  //   self.apply_theme(ctx);
  // }

  // pub fn settings_changed(&self) -> bool {
  //   matches!(self.theme, Value::Modified(_))
  //     || matches!(self.text_size, Value::Modified(_))
  // }

  pub fn get_theme(&self) -> Theme {
    get_value(&self.theme)
  }

  pub fn set_theme(&mut self, ctx: &Context, theme: Theme) {
    self.theme = Value::Modified(theme);
    self.apply_theme(ctx);
  }

  pub fn get_text_size(&self) -> TextSize {
    get_value(&self.text_size)
  }

  pub fn set_text_size(&mut self, ctx: &Context, text_size: u8) {
    self.text_size = Value::Modified(text_size);
    self.set_style(ctx);
  }

  pub fn get_numbers(&self) -> bool {
    get_value(&self.numbers)
  }

  pub fn toggle_numbers(&mut self) {
    self.numbers = Value::Modified(!self.get_numbers());
  }

  pub fn get_autoflip(&self) -> bool {
    !get_value(&self.auto_flip)
  }

  pub fn toggle_autoflip(&mut self) {
    self.auto_flip = Value::Modified(self.get_autoflip());
  }

  pub fn get_opponentflip(&self) -> bool {
    get_value(&self.opponent_flip)
  }

  pub fn toggle_opponentflip(&mut self) {
    self.opponent_flip = Value::Modified(!self.get_opponentflip());
  }

  pub fn get_advanced(&self) -> bool {
    !get_value(&self.advanced_settings)
  }

  pub fn toggle_advanced(&mut self) {
    self.advanced_settings = Value::Modified(self.get_advanced());
  }

  pub fn get_evalbar(&self) -> bool {
    !get_value(&self.eval_bar)
  }

  pub fn toggle_evalbar(&mut self) {
    self.eval_bar = Value::Modified(self.get_evalbar())
  }

  fn set_style(&self, ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let text_size = f32::from(get_value(&self.text_size));
    let font = FontId::proportional(text_size);
    style.spacing.icon_width = text_size * 0.7;
    style.spacing.icon_width_inner = text_size * 0.5;
    style.spacing.combo_height = 460.0;
    style.text_styles = [
      (TextStyle::Body, font.clone()),
      (TextStyle::Button, font.clone()),
      (TextStyle::Monospace, font),
    ]
    .into();
    ctx.set_style(style);
  }

  fn apply_theme(&self, ctx: &Context) {
    ctx.set_visuals(get_value(&self.theme).get_visuals());
  }
}

impl Parameter<Self> for Theme {
  fn default_value() -> Self {
    Self::Preset(PresetTheme::Dark)
  }
}

type TextSize = u8;

impl Parameter<Self> for TextSize {
  fn default_value() -> Self {
    24
  }
}

impl Parameter<Self> for bool {
  fn default_value() -> Self {
    true
  }
}
