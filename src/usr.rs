// src/usr.rs

use crate::{
  gem::{
    GemDoc, GemTag, GemText,
  },
  text::{
    Text, Doc,
  },
  screen::{
    self, Rect, Frame,
  },
  util::{
    self,
  },
};
use crossterm::{
  style::{
    Color
  },
  event::{
    KeyCode
  },
};
use std::{
  fs, io
};
use toml::{
  Table, Value,
};


pub fn parse_color(value: &Value) -> Result<Color, String> {
  if let Value::String(hex) = value {
    util::color_from_hex(&hex)

  } else {
    Err("fart".into())
  }
}

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
  pub layout:    UserLayout,
  pub keys:      UserKeys,
} 

impl Default for User {
  fn default() -> Self {
    Self {
      init_url: "gemini://datapulp.smol.pub/".into(),
      layout:    UserLayout::default(),
      keys:      UserKeys::default(),
    }
  }
}

impl User {

  pub fn get_doc(&self, gdoc: &GemDoc, rect: &Rect) -> Doc {
    let text = self.layout.gemtext_to_text(&gdoc.doc);
    let frame = self.layout.get_frame_from_rect(rect);
    Doc::new(text, &frame)
  }

  pub fn get_frame_from_rect(&self, rect: &Rect) -> Frame {
    self.layout.get_frame_from_rect(rect)
  }

  pub fn parse(text: &str) -> Self {

    let mut usr = Self::default();
    let table = text.parse::<Table>()
      .unwrap_or_default();

    if let Some(Value::String(s)) = table.get("init_url") {
      usr.init_url = s.into();
    }

    if let Some(v) = table.get("colors") {
      usr.layout = UserLayout::parse_module(v)
          .unwrap_or_default();
    }
    if let Some(v) = table.get("keys") {
      usr.keys = UserKeys::parse_module(v)
          .unwrap_or_default();
    }
    
    usr
  }
}

#[derive(Clone)]
pub struct UserKeys {
  pub global:      KeyCode,
  pub cancel:      KeyCode,
  pub load_usr:    KeyCode,
  pub msg_view:    KeyCode,
  pub tab_view:    KeyCode,
  pub move_up:     KeyCode,
  pub move_down:   KeyCode,
  pub move_left:   KeyCode,
  pub move_right:  KeyCode,
  pub cycle_left:  KeyCode,
  pub cycle_right: KeyCode,
  pub inspect:     KeyCode,
  pub delete_tab:  KeyCode,
  pub new_tab:     KeyCode,
  pub ack:         KeyCode, 
  pub yes:         KeyCode, 
  pub no:          KeyCode
} 

impl Default for UserKeys {
  fn default() -> Self {
    Self {
      global:      KeyCode::Char('g'),
      cancel:      KeyCode::Esc,
      load_usr:    KeyCode::Char('c'),
      msg_view:    KeyCode::Char('m'),
      tab_view:    KeyCode::Char('t'),
      move_up:     KeyCode::Up,
      move_down:   KeyCode::Down,
      move_left:   KeyCode::Left,
      move_right:  KeyCode::Right,
      cycle_left:  KeyCode::Char('E'),
      cycle_right: KeyCode::Char('N'),
      inspect:     KeyCode::Enter,
      delete_tab:  KeyCode::Char('d'),
      new_tab:     KeyCode::Char('n'),
      ack:         KeyCode::Enter, 
      yes:         KeyCode::Char('y'), 
      no:          KeyCode::Char('n')
    }
  }
}

impl UserModule for UserKeys {

  fn from_table(table: &Table) -> Result<Self, String> {

    let mut usr = Self::default();

    if let Some(Value::String(s)) = table.get("global") {
      if let Some(kc) = Self::key_from_string(s) {
        usr.global = kc;
      }
    }

    if let Some(Value::String(s)) = table.get("cancel") {
      if let Some(kc) = Self::key_from_string(s) {
        usr.cancel = kc;
      }
    }

    if let Some(Value::String(s)) = table.get("load_usr") {
      if let Some(kc) = Self::key_from_string(s) {
        usr.load_usr = kc;
      }
    }

    if let Some(Value::String(s)) = table.get("msg_view") {
      if let Some(kc) = Self::key_from_string(s) {
        usr.msg_view = kc;
      }
    }

    if let Some(Value::String(s)) = table.get("tab_view") {
      if let Some(kc) = Self::key_from_string(s) {
        usr.tab_view = kc;
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

#[derive(Debug)]
enum UserTextKey {
  Color(ColorK), 
  Usize(UsizeK), 
  Prefix,
}
impl UserTextKey {
  pub fn try_from_string(field: &str) 
    -> Result<Self, String> 
  {
    match field {
      "fg" => Ok(Self::Color(ColorK::Fg)),
      "bg" => Ok(Self::Color(ColorK::Bg)),
      "above" => Ok(Self::Usize(UsizeK::Above)),
      "below" => Ok(Self::Usize(UsizeK::Below)),
      "prefix" => Ok(Self::Prefix),
      field => 
        Err(format!("{} no such field in the table", field)),
    }
  }
}

#[derive(Debug)]
enum ColorK {
  Fg, Bg,
}
impl ColorK {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<Color, String>
  {
    parse_color(value)
      .map_err(|e| format!("{:?} : {}", self, e))
  }
}

#[derive(Debug)]
enum UsizeK {
  Above, Below,
}
impl UsizeK {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<usize, String>
  {
    match value {
      Value::Integer(i) => 
        usize::try_from(*i)
          .map_err(|e| format!("{:?} : {}", self, e)),
      v => 
        Err(format!("{:?} doesn't take {:?}", self, v)),
    }
  }
}

#[derive(Clone)]
pub struct UserText {
  pub fg: Option<Color>,
  pub bg: Option<Color>,
  pub above: usize,
  pub below: usize,
  pub prefix: String,
} 

impl Default for UserText {
  fn default() -> Self {
    Self {
      fg: None,
      bg: None,
      above: 0,
      below: 0,
      prefix: "".to_string(),
    }
  }
}

impl UserText {

  pub fn get_text(&self, text: &str) -> Text {
    let mut text = Text::from(text)
      .above(self.above)
      .below(self.below)
      .prefix(&self.prefix);

    if let Some(fg) = self.fg {
      text = text.fg(fg);
    }
    if let Some(bg) = self.bg {
      text = text.bg(bg);
    }

    text
  }

  pub fn try_assign(&mut self, key: &UserTextKey, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      UserTextKey::Color(k) => {
        let v = k.try_parse_value(&value)?;
        match k {
          ColorK::Fg => self.fg = Some(v),
          ColorK::Bg => self.bg = Some(v),
        }
        Ok(())
      }
      UserTextKey::Usize(k) => {
        let v = k.try_parse_value(&value)?;
        match k {
          UsizeK::Above => self.above = v,
          UsizeK::Below => self.below = v,
        }
        Ok(())
      }
      UserTextKey::Prefix => {
        if let Value::String(s) = value {
          self.prefix = s.into(); 
          Ok(())
        } else {
          Err(format!("prefix doesnt take {:?}", value))
        }
      }
    }
  }

  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = UserTextKey::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }

    Ok(self)
  }
}
#[derive(Debug)]
enum UserLayoutKey {
  Color(LayoutColorKey), 
  UserText(LayoutTextKey), 
  U16(LayoutU16Key),
}
impl UserLayoutKey {
  pub fn try_from_string(field: &str) 
    -> Result<Self, String> 
  {
    match field {
      "fg" => 
        Ok(Self::Color),
      "background" => 
        Ok(Self::Color),
      "above" => Ok(Self::UserTextKey(UsizeK::Above)),
      "below" => Ok(Self::UserTextKey(UsizeK::Below)),
      "prefix" => Ok(Self::U16),
      field => 
        Err(format!("{} no such field in the table", field)),
    }
  }
}

#[derive(Clone)]
pub struct UserLayout {
  pub x_text:    u16,
  pub y_text:    u16,
  pub x_page:    u16,
  pub y_page:    u16,
  pub scroll_at: u16,
  pub background: Color,
  pub banner:     Color,
  pub border:     Color,
  pub dialog:     Color,
  pub text:       UserText,
  pub heading1:   UserText,
  pub heading2:   UserText,
  pub heading3:   UserText,
  pub link:       UserText,
  pub badlink:    UserText,
  pub quote:      UserText,
  pub list:       UserText,
  pub preformat:  UserText,
} 

impl Default for UserLayout {
  fn default() -> Self {
    Self {
      scroll_at:  3,
      x_text:     0,
      y_text:     0,
      x_page:     0,
      y_page:     0,
      background: Color::Black,
      banner:     Color::White,
      border:     Color::White,
      dialog:     Color::White,
      text:       UserText::default(),
      heading1:   UserText::default(),
      heading2:   UserText::default(),
      heading3:   UserText::default(),
      link:       UserText::default(),
      badlink:    UserText::default(),
      quote:      UserText::default(),
      list:       UserText::default(),
      preformat:  UserText::default(),
    }
  }
}

impl UserModule for UserLayout {
  fn from_table(table: &Table) -> Result<Self, String> {

    let mut usr = Self::default();

    if let Some(Value::Integer(i)) = table.get("x_text") 
    {
      usr.x_text = u16::try_from(*i)
        .unwrap_or(0);
    }

    if let Some(Value::Integer(i)) = table.get("y_text") 
    {
      usr.y_text = u16::try_from(*i)
        .unwrap_or(0);
    }

    if let Some(Value::Integer(i)) = table.get("x_page") 
    {
      usr.x_page = u16::try_from(*i)
        .unwrap_or(0);
    }

    if let Some(Value::Integer(i)) = table.get("y_page") 
    {
      usr.y_page = u16::try_from(*i)
        .unwrap_or(0);
    }
    if let Some(Value::Integer(i)) = table.get("scroll_at") 
    {
      usr.scroll_at = u16::try_from(*i)
        .unwrap_or(0);
    }

    if let Some(c) = table.get("background")
                      .or(table.get("bg"))
                      .and_then(|v| parse_color(v))
    {
      usr.background = c;
    } 

    if let Some(c) = table.get("banner") 
                      .and_then(|v| parse_color(v))
    {
      usr.banner = c
    }

    if let Some(c) = table.get("border") 
                      .and_then(|v| parse_color(v))
    {
      usr.border = c;
    }

    if let Some(c) = table.get("dialog") 
                      .and_then(|v| parse_color(v))
    {
      usr.dialog = c;
    }

    if let Some(Value::Table(t)) = table.get("text") 
    {
      usr.text = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("heading1")
                                    .or(table.get("h1"))
    {
      usr.heading1 = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("heading2")
                                    .or(table.get("h2"))
    {
      usr.heading2 = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("heading3")
                                    .or(table.get("h3"))
    {
      usr.heading3 = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("badlink") 
    {
      usr.badlink = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("link") 
    {
      usr.link = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("quote") 
    {
      usr.quote = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("list") 
    {
      usr.list = 
        UserText::default().read_table(t);
    }

    if let Some(Value::Table(t)) = table.get("preformat") 
    {
      usr.preformat = 
        UserText::default().read_table(t);
    }

    Ok(usr)
  }
}

impl UserLayout {

  pub fn get_rect_from_dim(&self, w: u16, h: u16) -> Rect {
    Rect::new(w, h)
      .crop_x(self.x_page)
      .crop_y(self.y_page)
  }

  pub fn get_frame_from_rect(&self, rect: &Rect) -> Frame {
    Frame {
      inner: rect
        .crop_x(self.scroll_at)
        .crop_y(self.scroll_at),
      outer: rect.clone(),
    }
  }

  pub fn gemtext_to_text(&self, gem: &Vec<GemText>) 
    -> Vec<Text>
  {
    gem.iter()
      .map(|gem| self.get_user_text(&gem)).collect()
  }

  pub fn get_user_text(&self, gtxt: &GemText) -> Text {
    match gtxt.tag {
      GemTag::HeadingOne => 
        self.heading1.get_text(&gtxt.txt).wrap(),

      GemTag::HeadingTwo => 
        self.heading2.get_text(&gtxt.txt).wrap(),

      GemTag::HeadingThree => 
        self.heading3.get_text(&gtxt.txt).wrap(),

      GemTag::Text => 
        self.text.get_text(&gtxt.txt).wrap(),

      GemTag::PreFormat => 
        self.preformat.get_text(&gtxt.txt),

      GemTag::Link(_, _) => 
        self.link.get_text(&gtxt.txt).wrap(),

      GemTag::BadLink(_) => 
        self.badlink.get_text(&gtxt.txt),

      GemTag::ListItem => 
        self.list.get_text(&gtxt.txt).wrap(),

      GemTag::Quote => 
        self.quote.get_text(&gtxt.txt).wrap(),
    }
  }
}
