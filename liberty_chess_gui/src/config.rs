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

pub struct Configuration {
  theme: Value<Theme>,
  text_size: Value<TextSize>,
  numbers: Value<OptionalFeature>,
}

impl Configuration {
  pub fn new(ctx: &eframe::CreationContext) -> Self {
    let config = ctx.storage.as_ref().map_or(
      Self {
        theme: Value::Default,
        text_size: Value::Default,
        numbers: Value::Default,
      },
      |storage| Self {
        theme: load(storage.get_string(THEME_KEY)),
        text_size: load(storage.get_string(TEXT_SIZE_KEY)),
        numbers: load(storage.get_string(NUMBER_KEY)),
      },
    );
    config.set_style(&ctx.egui_ctx);
    config.apply_theme(&ctx.egui_ctx);

    config
  }

  pub fn save(&self, storage: &mut dyn Storage) {
    save(storage, THEME_KEY, &self.theme);
    save(storage, TEXT_SIZE_KEY, &self.text_size);
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

  pub fn get_numbers(&self) -> OptionalFeature {
    get_value(&self.numbers)
  }

  pub fn toggle_numbers(&mut self) {
    self.numbers = Value::Modified(!self.get_numbers());
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
type OptionalFeature = bool;

impl Parameter<Self> for OptionalFeature {
  fn default_value() -> Self {
    true
  }
}

type Frametime = u64;

impl Parameter<Self> for Frametime {
  fn default_value() -> Self {
    200
  }
}
