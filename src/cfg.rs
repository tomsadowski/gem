// src/cfg.rs

use crate::{
  gem::{GemDoc, GemType, GemText},
  text::{Text},
};
use crossterm::{
  style::{Color},
  event::{KeyCode},
};
use std::{fs, io};
use serde::Deserialize;
use toml::{
  Table, Value,
};


pub fn hex_from_char(u: char) -> Option<u8> {
  match u {
    '0' => Some(0),
    '1' => Some(1),
    '2' => Some(2),
    '3' => Some(3),
    '4' => Some(4),
    '5' => Some(5),
    '6' => Some(6),
    '7' => Some(7),
    '8' => Some(8),
    '9' => Some(9),
    'a' => Some(10),
    'b' => Some(11),
    'c' => Some(12),
    'd' => Some(13),
    'e' => Some(14),
    'f' => Some(15),
    _ => None,
  }
}

pub fn parse_next_u8<I>(v: &mut I) -> Option<u8> 
where I: Iterator<Item = char>
{
  v
    .next()
    .and_then(|c| hex_from_char(c))
    .zip(
      v
        .next()
        .and_then(|c| hex_from_char(c)))
    .map(|(a, b)| 16 * a + b)
}

pub fn color_from_hex(text: &str) -> Option<Color> {
  let mut c = text.chars();

  let r = parse_next_u8(&mut c);
  let g = parse_next_u8(&mut c);
  let b = parse_next_u8(&mut c);

  if let (Some(r), Some(g), Some(b)) = (r, g, b) {
    Some(Color::Rgb {r, g, b})

  } else {
    None
  }
}

// Either a table or a path to a file containing a table
pub trait ConfigModule: Sized {

  // required
  fn from_table(table: &Table) -> Result<Self, String>;

  // provided
  fn parse_module(value: &Value) -> Result<Self, String> {
    match value {

      Value::Table(t) => {
        Self::from_table(t)
      }

      // stuff all errors into a string
      Value::String(s) => {
        fs::read_to_string(s)
          .map_err(|e| e.to_string())
          .and_then(|txt| Table::try_from(&txt)
            .map_err(|e| e.to_string()))
          .and_then(|table| Self::from_table(&table))
      }

      v => {
        Err(format!("did not expect value of type {}", v))
      },
    }
  }
}

#[derive(Clone)]
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

  pub fn parse(text: &str) -> Self {

    let mut cfg = Self::default();
    let table = Table::try_from(text).unwrap_or_default();

    if let Some(v) = table.get("colors") {
      cfg.colors = 
        ColorParams::parse_module(v)
          .unwrap_or_default();
    }
    if let Some(v) = table.get("keys") {
      cfg.keys = 
        KeyParams::parse_module(v)
          .unwrap_or_default();
    }
    if let Some(v) = table.get("format") {
      cfg.format = 
        FormatParams::parse_module(v)
          .unwrap_or_default();
    }
    
    cfg
  }
}

#[derive(Clone)]
pub struct KeyParams {
  pub global:     KeyCode,
  pub load_cfg:   KeyCode,
  pub msg_view:   KeyCode,
  pub tab_view:   KeyCode,

  pub move_up:      KeyCode,
  pub move_down:    KeyCode,
  pub move_left:    KeyCode,
  pub move_right:   KeyCode,
  pub cycle_left:   KeyCode,
  pub cycle_right:  KeyCode,
  pub inspect:      KeyCode,
  pub delete_tab:   KeyCode,
  pub new_tab:      KeyCode,

  pub ack:    KeyCode, 
  pub yes:    KeyCode, 
  pub no:     KeyCode
} 

impl Default for KeyParams {
  fn default() -> Self {
    Self {
      global:     KeyCode::Char('g'),
      load_cfg:   KeyCode::Char('c'),
      msg_view:   KeyCode::Char('m'),
      tab_view:   KeyCode::Char('t'),

      move_up:      KeyCode::Char('o'),
      move_down:    KeyCode::Char('i'),
      move_left:    KeyCode::Char('e'),
      move_right:   KeyCode::Char('n'),
      cycle_left:   KeyCode::Char('E'),
      cycle_right:  KeyCode::Char('N'),
      inspect:      KeyCode::Char('w'),
      delete_tab:   KeyCode::Char('v'),
      new_tab:      KeyCode::Char('p'),

      ack:    KeyCode::Char('y'), 
      yes:    KeyCode::Char('y'), 
      no:     KeyCode::Char('n')
    }
  }
}

impl ConfigModule for KeyParams {
  fn from_table(table: &Table) -> Result<Self, String> {
    let cfg = Self::default();
    if let Some(v) = table.get("colors") {
    }
    Ok(cfg)
  }
}

#[derive(Clone)]
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

impl ConfigModule for FormatParams {
  fn from_table(table: &Table) -> Result<Self, String> {
    let cfg = Self::default();
    if let Some(v) = table.get("colors") {
    }
    Ok(cfg)
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

#[derive(Clone)]
pub struct ColorParams {
  pub background: Color,
  pub banner:     Color,
  pub border:     Color,
  pub dialog:     Color,
  pub text:       Color,
  pub heading1:   Color,
  pub heading2:   Color,
  pub heading3:   Color,
  pub link:       Color,
  pub badlink:    Color,
  pub quote:      Color,
  pub list:       Color,
  pub preformat:  Color,
} 

impl Default for ColorParams {
  fn default() -> Self {
    Self {
      background: Color::Black,
      border:     Color::Grey,
      badlink:    Color::Grey,
      banner:     Color::Grey,
      dialog:     Color::White,
      text:       Color::White,
      list:       Color::White,
      heading1:   Color::White,
      heading2:   Color::White,
      heading3:   Color::White,
      link:       Color::White,
      quote:      Color::White,
      preformat:  Color::White,
    }
  }
}

impl ColorParams {
  pub fn parse_color(value: &Value) -> Option<Color> {
    if let Value::String(hex) = value {
      color_from_hex(&hex)
    } else {
      None
    }
  }
  pub fn from_gem_tag(&self, tag: &GemType) -> Color {
    match tag {
      _ => Color::White,
    }
  }
}

impl ConfigModule for ColorParams {
  fn from_table(table: &Table) -> Result<Self, String> {
    let mut cfg = Self::default();

    if let Some(c) = 
      table.get("background")
        .or(table.get("bg"))
        .and_then(|v| Self::parse_color(v))
    {
      cfg.background = c;
    } 
    if let Some(c) = 
      table.get("banner") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.banner = c
    }
    if let Some(c) = 
      table.get("border") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.border = c;
    }
    if let Some(c) = 
      table.get("dialog") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.dialog = c;
    }
    if let Some(c) = 
      table.get("text") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.text = c;
    }
    if let Some(c) = 
      table.get("heading1") 
        .or(table.get("h1"))
        .and_then(|v| Self::parse_color(v))
    {
      cfg.heading1 = c;
    }
    if let Some(c) = 
      table.get("heading2") 
        .or(table.get("h2"))
        .and_then(|v| Self::parse_color(v))
    {
      cfg.heading2 = c;
    }
    if let Some(c) = 
      table.get("heading3") 
        .or(table.get("h3"))
        .and_then(|v| Self::parse_color(v))
    {
      cfg.heading3 = c;
    }
    if let Some(c) = 
      table.get("badlink") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.badlink = c;
    }
    if let Some(c) = 
      table.get("link") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.link = c;
    }
    if let Some(c) = 
      table.get("quote") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.quote = c;
    }
    if let Some(c) = 
      table.get("list") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.list = c;
    }
    if let Some(c) = 
      table.get("preformat") 
        .and_then(|v| Self::parse_color(v))
    {
      cfg.preformat = c;
    }

    Ok(cfg)
  }
}
