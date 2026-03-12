// src/usr.rs

use crate::{
  gem::{GemDoc, GemTag, GemText},
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


// Either a module or a path to a file containing a module
pub trait UserModule: Sized {

  // required
  fn from_table(table: &Table) -> Result<Self, String>;

  // provided
  fn parse_module(value: &Value) -> Result<Self, String> {
    match value {

      // parse module from table
      Value::Table(module) => {
        Self::from_table(module)
      }

      // try to parse module from file
      Value::String(module_path) => {
        fs::read_to_string(module_path)
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
pub struct User {
  pub init_url:  String,
  pub scroll_at: u16,
  pub colors:    UserColors,
  pub keys:      UserKeys,
  pub format:    UserFormat,
} 

impl Default for User {
  fn default() -> Self {
    Self {
      init_url: "gemini://datapulp.smol.pub/".into(),
      scroll_at: 3,
      colors:    UserColors::default(),
      keys:      UserKeys::default(),
      format:    UserFormat::default(),
    }
  }
}

impl User {

  pub fn gemdoc_to_text(&self, doc: &GemDoc) -> Vec<Text> {
    doc.doc.iter()
      .map(|gem_text| self.gemtext_to_text(&gem_text))
      .collect()
  }

  pub fn gemtext_to_text(&self, gem: &GemText) 
    -> Text 
  {
    let (before, after) = 
      self.format.from_gem_tag(&gem.tag);

    let text = 
      Text::from(gem.txt.as_str())
        .fg(self.colors.from_gem_tag(&gem.tag))
        .bg(self.colors.background)
        .before(before.into())
        .after(after.into());

    if let GemTag::PreFormat = gem.tag {
      text

    } else {
      text.wrap()
    }
  }

  pub fn parse(text: &str) -> Self {

    let mut usr = Self::default();
    let table = text.parse::<Table>()
      .unwrap_or_default();

    if let Some(Value::String(s)) = table.get("init_url") {
      usr.init_url = s.into();
    }

    if let Some(Value::Integer(i)) = table.get("scroll_at") 
    {
      usr.scroll_at = u16::try_from(*i)
        .unwrap_or(0);
    }

    if let Some(v) = table.get("colors") {
      usr.colors = UserColors::parse_module(v)
          .unwrap_or_default();
    }
    if let Some(v) = table.get("keys") {
      usr.keys = UserKeys::parse_module(v)
          .unwrap_or_default();
    }
    if let Some(v) = table.get("format") {
      usr.format = UserFormat::parse_module(v)
          .unwrap_or_default();
    }
    
    usr
  }
}

#[derive(Clone)]
pub struct UserKeys {
  pub global:     KeyCode,
  pub cancel:     KeyCode,
  pub load_usr:   KeyCode,
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

impl Default for UserKeys {
  fn default() -> Self {
    Self {
      global:     KeyCode::Char('g'),
      cancel:     KeyCode::Esc,
      load_usr:   KeyCode::Char('c'),
      msg_view:   KeyCode::Char('m'),
      tab_view:   KeyCode::Char('t'),

      move_up:      KeyCode::Up,
      move_down:    KeyCode::Down,
      move_left:    KeyCode::Left,
      move_right:   KeyCode::Right,
      cycle_left:   KeyCode::Char('E'),
      cycle_right:  KeyCode::Char('N'),
      inspect:      KeyCode::Enter,
      delete_tab:   KeyCode::Char('d'),
      new_tab:      KeyCode::Char('n'),

      ack:    KeyCode::Enter, 
      yes:    KeyCode::Char('y'), 
      no:     KeyCode::Char('n')
    }
  }
}

impl UserModule for UserKeys {

  fn from_table(table: &Table) -> Result<Self, String> {

    let mut usr = Self::default();

    if let Some(Value::String(s)) = table.get("global") {
      if let Some(k) = Self::key_from_string(s) {
        usr.global = k;
      }
    }
    if let Some(Value::String(s)) = table.get("cancel") {
      if let Some(k) = Self::key_from_string(s) {
        usr.cancel = k;
      }
    }
    if let Some(Value::String(s)) = table.get("load_usr") {
      if let Some(k) = Self::key_from_string(s) {
        usr.load_usr = k;
      }
    }
    if let Some(Value::String(s)) = table.get("msg_view") {
      if let Some(k) = Self::key_from_string(s) {
        usr.msg_view = k;
      }
    }
    if let Some(Value::String(s)) = table.get("tab_view") {
      if let Some(k) = Self::key_from_string(s) {
        usr.tab_view = k;
      }
    }
    if let Some(Value::String(s)) = table.get("move_up") {
      if let Some(k) = Self::key_from_string(s) {
        usr.move_up = k;
      }
    }
    if let Some(Value::String(s)) = table.get("move_down") {
      if let Some(k) = Self::key_from_string(s) {
        usr.move_down = k;
      }
    }
    if let Some(Value::String(s)) = table.get("move_left") {
      if let Some(k) = Self::key_from_string(s) {
        usr.move_left = k;
      }
    }
    if let Some(Value::String(s)) = table.get("move_right") {
      if let Some(k) = Self::key_from_string(s) {
        usr.move_right = k;
      }
    }
    if let Some(Value::String(s)) = table.get("cycle_left") {
      if let Some(k) = Self::key_from_string(s) {
        usr.cycle_left = k;
      }
    }
    if let Some(Value::String(s)) = table.get("cycle_right") {
      if let Some(k) = Self::key_from_string(s) {
        usr.cycle_right = k;
      }
    }
    if let Some(Value::String(s)) = table.get("inspect") {
      if let Some(k) = Self::key_from_string(s) {
        usr.inspect = k;
      }
    }
    if let Some(Value::String(s)) = table.get("delete_tab") {
      if let Some(k) = Self::key_from_string(s) {
        usr.delete_tab = k;
      }
    }
    if let Some(Value::String(s)) = table.get("new_tab") {
      if let Some(k) = Self::key_from_string(s) {
        usr.new_tab = k;
      }
    }
    if let Some(Value::String(s)) = table.get("ack") {
      if let Some(k) = Self::key_from_string(s) {
        usr.ack = k;
      }
    }
    if let Some(Value::String(s)) = table.get("yes") {
      if let Some(k) = Self::key_from_string(s) {
        usr.yes = k;
      }
    }
    if let Some(Value::String(s)) = table.get("no") {
      if let Some(k) = Self::key_from_string(s) {
        usr.no = k;
      }
    }
    Ok(usr)
  }
}

impl UserKeys {
  pub fn key_from_string(text: &str) -> Option<KeyCode> {
    match text {
      "esc" | "escape" => 
        Some(KeyCode::Esc),

      "ent" | "enter" => 
        Some(KeyCode::Enter),

      "left" => 
        Some(KeyCode::Left),

      "up" => 
        Some(KeyCode::Up),

      "down" => 
        Some(KeyCode::Down),

      "right" => 
        Some(KeyCode::Right),

      t => 
        t
          .chars()
          .next()
          .map(|c| KeyCode::Char(c)),
    }
  }
}

#[derive(Clone)]
pub struct UserFormat {
  pub x_margin:    u16,
  pub y_margin:    u16,
  pub list_prefix: String,
  pub heading1:    (u8, u8),
  pub heading2:    (u8, u8),
  pub heading3:    (u8, u8),
} 

impl Default for UserFormat {

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

impl UserModule for UserFormat {

  fn from_table(table: &Table) -> Result<Self, String> {

    let usr = Self::default();

    if let Some(v) = 
      table.get("heading1")
        .or(table.get("h1"))
    {
    }
    if let Some(v) = 
      table.get("heading2")
        .or(table.get("h2"))
    {
    }
    if let Some(v) = 
      table.get("heading1")
        .or(table.get("h1"))
    {
    }

    Ok(usr)
  }
}

impl UserFormat {

  pub fn from_gem_tag(&self, gem: &GemTag) -> (u8, u8) {
    match gem {
      GemTag::HeadingOne    => self.heading1,
      GemTag::HeadingTwo    => self.heading2,
      GemTag::HeadingThree  => self.heading3,
      _ => (0, 0)
    }
  }
}

#[derive(Clone)]
pub struct UserColors {
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

impl Default for UserColors {
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

impl UserColors {

  fn hex_from_char(u: char) -> Option<u8> {
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

  fn parse_next_u8<I>(v: &mut I) -> Option<u8> 
  where I: Iterator<Item = char>
  {
    let a = v.next().and_then(|c| Self::hex_from_char(c));
    let b = v.next().and_then(|c| Self::hex_from_char(c));
    a.zip(b).map(|(a, b)| 16 * a + b)
  }

  fn color_from_hex(text: &str) -> Option<Color> {
    let mut c = text.chars();

    let r = Self::parse_next_u8(&mut c);
    let g = Self::parse_next_u8(&mut c);
    let b = Self::parse_next_u8(&mut c);

    if let (Some(r), Some(g), Some(b)) = (r, g, b) {
      Some(Color::Rgb {r, g, b})

    } else {
      None
    }
  }

  pub fn parse_color(value: &Value) -> Option<Color> {
    if let Value::String(hex) = value {
      Self::color_from_hex(&hex)
    } else {
      None
    }
  }

  pub fn from_gem_tag(&self, tag: &GemTag) -> Color {
    match tag {
      GemTag::HeadingOne    => self.heading1,
      GemTag::HeadingTwo    => self.heading2,
      GemTag::HeadingThree  => self.heading3,
      GemTag::Text          => self.text,
      GemTag::PreFormat     => self.preformat,
      GemTag::Link(_, _)    => self.link,
      GemTag::BadLink(_)    => self.badlink,
      GemTag::ListItem      => self.list,
      GemTag::Quote         => self.quote,
    }
  }
}

impl UserModule for UserColors {
  fn from_table(table: &Table) -> Result<Self, String> {
    let mut usr = Self::default();

    if let Some(c) = 
      table.get("background")
        .or(table.get("bg"))
        .and_then(|v| Self::parse_color(v))
    {
      usr.background = c;
    } 
    if let Some(c) = 
      table.get("banner") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.banner = c
    }
    if let Some(c) = 
      table.get("border") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.border = c;
    }
    if let Some(c) = 
      table.get("dialog") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.dialog = c;
    }
    if let Some(c) = 
      table.get("text") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.text = c;
    }
    if let Some(c) = 
      table.get("heading1") 
        .or(table.get("h1"))
        .and_then(|v| Self::parse_color(v))
    {
      usr.heading1 = c;
    }
    if let Some(c) = 
      table.get("heading2") 
        .or(table.get("h2"))
        .and_then(|v| Self::parse_color(v))
    {
      usr.heading2 = c;
    }
    if let Some(c) = 
      table.get("heading3") 
        .or(table.get("h3"))
        .and_then(|v| Self::parse_color(v))
    {
      usr.heading3 = c;
    }
    if let Some(c) = 
      table.get("badlink") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.badlink = c;
    }
    if let Some(c) = 
      table.get("link") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.link = c;
    }
    if let Some(c) = 
      table.get("quote") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.quote = c;
    }
    if let Some(c) = 
      table.get("list") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.list = c;
    }
    if let Some(c) = 
      table.get("preformat") 
        .and_then(|v| Self::parse_color(v))
    {
      usr.preformat = c;
    }

    Ok(usr)
  }
}
