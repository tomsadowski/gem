// src/cfg.rs

use crate::{
  gem::{GemDoc, GemType, GemText},
  text::{Text},
};
use crossterm::{
  style::{Color},
};
use std::fs;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
  pub init_url:  String,
  pub scroll_at: u16,
  pub colors:    ColorParams,
  pub keys:      KeyParams,
  pub format:    FormatParams,
} 

impl Default for Config {
  fn default() -> Self {
    Self {
      init_url: "gemini://datapulp.smol.pub/".into(),
      scroll_at: 3,
      colors:    ColorParams::default(),
      keys:      KeyParams::default(),
      format:    FormatParams::default(),
    }
  }
}

impl Config {

  pub fn gemdoc_to_text(&self, doc: &GemDoc) -> Vec<Text> {
    doc.doc.iter()
      .map(|gem_text| self.gemtext_to_text(&gem_text))
      .collect()
  }

  pub fn gemtext_to_text(&self, gem: &GemText) 
    -> Text 
  {
    let (before, after) = 
      self.format.from_gem_type(&gem.tag);

    let text = 
      Text::from(gem.txt.as_str())
        .fg(self.colors.from_gem_type(&gem.tag))
        .before(before.into())
        .after(after.into());

    if let GemType::PreFormat = gem.tag {
      text

    } else {
      text.wrap()
    }
  }

  pub fn parse(text: &str) -> Result<Self, String> {
    toml::from_str(text).map_err(|e| e.to_string())
  }
}

#[derive(Clone, Deserialize)]
pub struct KeyParams {
  pub global:     char,
  pub load_cfg:   char,
  pub msg_view:   char,
  pub tab_view:   char,
  pub dialog:     DialogKeyParams,
  pub tab:        TabKeyParams,
} 
impl Default for KeyParams {
  fn default() -> Self {
    Self {
      global:     'g',
      load_cfg:   'c',
      msg_view:   'm',
      tab_view:   't',
      dialog:     DialogKeyParams::default(),
      tab:        TabKeyParams::default(),
    }
  }
}

#[derive(Clone, Deserialize)]
pub struct TabKeyParams {
  pub move_up:      char,
  pub move_down:    char,
  pub move_left:    char,
  pub move_right:   char,
  pub cycle_left:   char,
  pub cycle_right:  char,
  pub inspect:      char,
  pub delete_tab:   char,
  pub new_tab:      char,
} 
impl Default for TabKeyParams {
  fn default() -> Self {
    Self {
      move_up:      'o',
      move_down:    'i',
      move_left:    'e',
      move_right:   'n',
      cycle_left:   'E',
      cycle_right:  'N',
      inspect:      'w',
      delete_tab:   'v',
      new_tab:      'p',
    }
  }
}

#[derive(Clone, Deserialize)]
pub struct DialogKeyParams {
  pub ack:    char, 
  pub yes:    char, 
  pub no:     char
} 
impl Default for DialogKeyParams {
  fn default() -> Self {
    Self {
      ack:    'y', 
      yes:    'y', 
      no:     'n'
    }
  }
}

#[derive(Clone, Deserialize)]
pub struct FormatParams {
  pub x_margin:    u16,
  pub y_margin:    u16,
  pub list_prefix: String,
  pub heading1:    (u8, u8),
  pub heading2:    (u8, u8),
  pub heading3:    (u8, u8),
} 

impl Default for FormatParams {
  fn default() -> Self {
    Self {
      x_margin:    4,
      y_margin:    2,
      list_prefix: "- ".into(),
      heading1:    (3, 2),
      heading2:    (2, 1),
      heading3:    (1, 0),
    }
  }
}

impl FormatParams {

  pub fn from_gem_type(&self, gem: &GemType) -> (u8, u8) {
    match gem {
      GemType::HeadingOne => 
        self.heading1,
      GemType::HeadingTwo => 
        self.heading2,
      GemType::HeadingThree => 
        self.heading3,
      _ => (0, 0)
    }
  }
}

#[derive(Clone, Deserialize)]
pub struct ColorParams {
  pub background: (u8, u8, u8),
  pub banner:     (u8, u8, u8),
  pub border:     (u8, u8, u8),
  pub dialog:     (u8, u8, u8),
  pub text:       (u8, u8, u8),
  pub heading1:   (u8, u8, u8),
  pub heading2:   (u8, u8, u8),
  pub heading3:   (u8, u8, u8),
  pub link:       (u8, u8, u8),
  pub badlink:    (u8, u8, u8),
  pub quote:      (u8, u8, u8),
  pub list:       (u8, u8, u8),
  pub preformat:  (u8, u8, u8),
} 
impl Default for ColorParams {
  fn default() -> Self {
    Self {
      background: ( 40,  40,  40),
      border:     (128, 136, 144),
      badlink:    (128, 136, 144),
      banner:     (224, 240, 255),
      dialog:     (224, 240, 255),
      text:       (224, 240, 255),
      list:       (224, 240, 255),
      heading1:   (255, 144, 176),
      heading2:   (255, 144, 176),
      heading3:   (255, 144, 176),
      link:       (128, 255, 208),
      quote:      (255, 208, 160),
      preformat:  (255, 208, 160),
    }
  }
}
impl ColorParams {
  pub fn get_banner(&self) -> Color {
    let (r, g, b) = self.banner; 
    Color::Rgb {r, g, b}
  }

  pub fn get_dialog(&self) -> Color {
    let (r, g, b) = self.dialog; 
    Color::Rgb {r, g, b}
  }

  pub fn get_background(&self) -> Color {
    let (r, g, b) = self.background; 
    Color::Rgb {r, g, b}
  }

  pub fn from_gem_type(&self, gem: &GemType) -> Color {
    let (r, g, b) = match gem {
      GemType::HeadingOne => 
        self.heading1,
      GemType::HeadingTwo => 
        self.heading2,
      GemType::HeadingThree => 
        self.heading3,
      GemType::Text => 
        self.text,
      GemType::Quote => 
        self.quote,
      GemType::ListItem => 
        self.list,
      GemType::PreFormat => 
        self.preformat,
      GemType::Link(_, _) => 
        self.link,
      GemType::BadLink(_) => 
        self.badlink,
    };
    Color::Rgb {r, g, b}
  }
}
