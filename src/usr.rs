// src/usr.rs

use crate::{
  text::{TextPlane, Planar, Linear},
  screen::{Rect, PlaneView},
  widget::{TextBox},
};
use crossterm::{
  style::Color,
  event::KeyCode,
};
use toml::{Table, Value};

// module: usr
// a)   Parse '.gemset' file.
// b)   Help construct items
//      that require many user parameters.
// (a) read file, (b) co-author runtime data.

pub fn parse_color(value: &Value) -> Result<Color, String> {
  if let Value::String(hex) = value {
    color_from_hex(&hex)
  } else {
    Err("could not parse color from value".into())
  }
}
pub fn try_hex_from_char(u: char) -> Option<u8> {
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
pub fn try_next_u8<I>(v: &mut I) -> Option<u8> 
where I: Iterator<Item = char>
{
  let a = v.next().and_then(|c| try_hex_from_char(c));
  let b = v.next().and_then(|c| try_hex_from_char(c));
  a.zip(b).map(|(a, b)| 16 * a + b)
}
pub fn color_from_hex(text: &str) -> Result<Color, String> {
  let mut c = text.chars();
  let r = try_next_u8(&mut c);
  let g = try_next_u8(&mut c);
  let b = try_next_u8(&mut c);
  match (r, g, b) {
    (Some(r), Some(g), Some(b)) => {
      Ok(Color::Rgb {r, g, b})
    }
    _ => {
      Err("this... is not hex".into())
    }
  }
}
pub trait Field: Sized {
  fn try_from_string(s: &str) -> Result<Self, String>;
}
pub trait UserMod<F: Field>: Sized {
  fn try_assign(&mut self, field: &F, value: &Value) -> Result<(), String>;

  fn read_table(mut self, table: &Table) -> Result<Self, String> {
    for (key, value) in table.iter() {
      let field = F::try_from_string(&key)?;
      self.try_assign(&field, value)?;
    }
    Ok(self)
  }
}

#[derive(Debug)]
enum UserField {
  InitPath,
  Layout,
  Fields,
}
impl Field for UserField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "init_url" => Ok(Self::InitPath),
      "layout"   => Ok(Self::Layout),
      "keys"     => Ok(Self::Fields),
      s => 
        Err(format!("No field {} in User table", s)),
    }
  }
}

#[derive(Clone)]
pub struct User {
  pub init_path: String,
  pub layout:    UserLayout,
  pub keys:      UserKeys,
} 
impl Default for User {
  fn default() -> Self {
    Self {
      init_path: ".".into(),
      layout:    UserLayout::default(),
      keys:      UserKeys::default(),
    }
  }
}
impl Field for User {
  fn try_from_string(s: &str) -> Result<Self, String> {
    let table = s.parse::<Table>()
      .map_err(|e| e.to_string())?;
    Self::default().read_table(&table)
  }
}
impl UserMod<UserField> for User {
  fn try_assign(&mut self, field: &UserField, value: &Value) 
    -> Result<(), String> 
  {
    match field {
      UserField::InitPath => {
        if let Value::String(s) = value {
          self.init_path = s.into();
        } else {
          return Err("init_path field expects a string value".into())
        }
      }
      UserField::Layout => {
        if let Value::Table(t) = value {
          self.layout = UserLayout::default()
            .read_table(t)?;
        } else {
          return Err("layout field expects a table value".into())
        }
      }
      UserField::Fields => {
        if let Value::Table(t) = value {
          self.keys = UserKeys::default().read_table(t)?;
        } else {
          return Err("keys field expects a table value".into())
        }
      }
    }
    Ok(())
  }
}

#[derive(Debug)]
enum KeysField {
  Global, 
  LoadUser,
  MsgView, 
  TabView, 
  MoveUp, 
  MoveDown, 
  MoveLeft, 
  MoveRight,
  CycleLeft, 
  CycleRight, 
  DelTab, 
  NewTab, 
  Inspect, 
  Ack, 
  Yes, 
  No, 
  Cancel,
}
impl Field for KeysField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "global"      => Ok(Self::Global),
      "msg_view"    => Ok(Self::MsgView),
      "tab_view"    => Ok(Self::TabView),
      "load_usr"    => Ok(Self::LoadUser),
      "move_up"     => Ok(Self::MoveUp),
      "move_down"   => Ok(Self::MoveDown),
      "move_left"   => Ok(Self::MoveLeft),
      "move_right"  => Ok(Self::MoveRight),
      "cycle_left"  => Ok(Self::CycleLeft),
      "cycle_right" => Ok(Self::CycleRight),
      "delete_tab"  => Ok(Self::DelTab),
      "new_tab"     => Ok(Self::NewTab),
      "inspect"     => Ok(Self::Inspect),
      "ack"         => Ok(Self::Ack),
      "yes"         => Ok(Self::Yes),
      "no"          => Ok(Self::No),
      "cancel"      => Ok(Self::Cancel),
      s => 
        Err(format!("No field {} in UserKeys table", s)),
    }
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
impl UserMod<KeysField> for UserKeys {
  fn try_assign(&mut self, field: &KeysField, value: &Value) 
    -> Result<(), String> 
  {
    let v = Self::try_from_value(value)?;
    match field {
      KeysField::Global     => self.global = v,
      KeysField::MsgView    => self.msg_view = v,
      KeysField::LoadUser   => self.load_usr = v,
      KeysField::TabView    => self.tab_view = v,
      KeysField::MoveUp     => self.move_up = v,
      KeysField::MoveDown   => self.move_down = v,
      KeysField::MoveLeft   => self.move_left = v,
      KeysField::MoveRight  => self.move_right = v,
      KeysField::CycleLeft  => self.cycle_left = v,
      KeysField::CycleRight => self.cycle_right = v,
      KeysField::DelTab     => self.delete_tab = v,
      KeysField::NewTab     => self.new_tab = v,
      KeysField::Inspect    => self.inspect = v,
      KeysField::Ack        => self.ack = v,
      KeysField::Yes        => self.yes = v,
      KeysField::No         => self.no = v,
      KeysField::Cancel     => self.cancel = v,
    }
    Ok(())
  }
}
impl UserKeys {
  pub fn try_from_value(value: &Value) -> Result<KeyCode, String> {
    if let Value::String(s) = value {
      if let Some(keycode) = Self::keycode_from_string(&s) {
        Ok(keycode)
      } else {
        Err("could not parse keycode from string".into())
      }
    } else {
      Err("could not parse keycode from value".into())
    }
  }
  fn keycode_from_string(s: &str) -> Option<KeyCode> {
    match s {
      "esc" | "escape"  => Some(KeyCode::Esc),
      "ent" | "enter"   => Some(KeyCode::Enter),
      "space"           => Some(KeyCode::Char(' ')),
      "left"            => Some(KeyCode::Left),
      "up"              => Some(KeyCode::Up),
      "down"            => Some(KeyCode::Down),
      "right"           => Some(KeyCode::Right),
      s => 
        s.chars().next().map(|c| KeyCode::Char(c)),
    }
  }
}

#[derive(Debug)]
enum LayoutField {
  Color(ColorLayoutField), 
  U16(U16LayoutField),
}
impl Field for LayoutField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "x_text" => 
        Ok(Self::U16(U16LayoutField::XText)),
      "y_text" => 
        Ok(Self::U16(U16LayoutField::YText)),
      "x_page" => 
        Ok(Self::U16(U16LayoutField::XPage)),
      "y_page" => 
        Ok(Self::U16(U16LayoutField::YPage)),
      "scroll_at" => 
        Ok(Self::U16(U16LayoutField::ScrollAt)),
      "background" | "bg" => 
        Ok(Self::Color(ColorLayoutField::Bg)),
      "banner" => 
        Ok(Self::Color(ColorLayoutField::Banner)),
      s => 
        Err(format!("Layout table does not contain field {}.", s)),
    }
  }
}

#[derive(Debug)]
enum ColorLayoutField {
  Bg, Banner, Border, Dlg,
}
impl ColorLayoutField {
  pub fn try_parse_value(&self, value: &Value) -> Result<Color, String> {
    parse_color(value).map_err(|e| format!("{:?} : {}", self, e))
  }
}

#[derive(Debug)]
enum U16LayoutField {
  XPage, YPage, XText, YText, ScrollAt,
}
impl U16LayoutField {
  pub fn try_parse_value(&self, value: &Value) -> Result<u16, String> {
    if let Value::Integer(t) = value {
      u16::try_from(*t).map_err(|e| format!("{:?} : {}", self, e))
    } else {
      Err(format!("prefix doesnt take {:?}", value))
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
  pub background: Option<Color>,
  pub banner:     Option<Color>,
  pub border:     Option<Color>,
  pub dialog:     Option<Color>,
  pub text:       UserText,
} 
impl Default for UserLayout {
  fn default() -> Self {
    Self {
      scroll_at:  3,
      x_text:     0,
      y_text:     0,
      x_page:     0,
      y_page:     0,
      background: None,
      banner:     None,
      border:     None,
      dialog:     None,
      text:       UserText::default(),
    }
  }
}
impl UserMod<LayoutField> for UserLayout {
  fn try_assign(&mut self, field: &LayoutField, value: &Value) 
    -> Result<(), String> 
  {
    match field {
      LayoutField::Color(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          ColorLayoutField::Bg      => self.background = Some(v),
          ColorLayoutField::Banner  => self.banner = Some(v),
          ColorLayoutField::Border  => self.border = Some(v),
          ColorLayoutField::Dlg     => self.dialog = Some(v),
        }
      }
      LayoutField::U16(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          U16LayoutField::XText     => self.x_text = v,
          U16LayoutField::YText     => self.y_text = v,
          U16LayoutField::XPage     => self.x_page = v,
          U16LayoutField::YPage     => self.y_page = v,
          U16LayoutField::ScrollAt  => self.scroll_at = v,
        }
      }
    }
    Ok(())
  }
}
impl UserLayout {
  pub fn get_rect_from_dim(&self, w: u16, h: u16) -> Rect {
    Rect::new(w, h)
      .crop_x(self.x_page)
      .crop_y(self.y_page)
  }
}

#[derive(Debug)]
enum TextField {
  Color(ColorTextField), 
  Usize(UsizeTextField), 
  Prefix,
}
impl Field for TextField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "fg"      => Ok(Self::Color(ColorTextField::Fg)),
      "bg"      => Ok(Self::Color(ColorTextField::Bg)),
      "above"   => Ok(Self::Usize(UsizeTextField::Above)),
      "below"   => Ok(Self::Usize(UsizeTextField::Below)),
      "prefix"  => Ok(Self::Prefix),
      s => Err(format!("{} no such field in the table", s)),
    }
  }
}

#[derive(Debug)]
enum ColorTextField {
  Fg, Bg,
}
impl ColorTextField {
  pub fn try_parse_value(&self, value: &Value) -> Result<Color, String> {
    parse_color(value).map_err(|e| format!("{:?} : {}", self, e))
  }
}

#[derive(Debug)]
enum UsizeTextField {
  Above, Below,
}
impl UsizeTextField {
  pub fn try_parse_value(&self, value: &Value) -> Result<usize, String> {
    match value {
      Value::Integer(i) => 
        usize::try_from(*i).map_err(|e| format!("{:?} : {}", self, e)),
      value => 
        Err(format!("{:?} doesn't take {:?}", self, value)),
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
impl UserMod<TextField> for UserText {
  fn try_assign(&mut self, field: &TextField, value: &Value) 
    -> Result<(), String> 
  {
    match field {
      TextField::Color(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          ColorTextField::Fg => self.fg = Some(v),
          ColorTextField::Bg => self.bg = Some(v),
        }
      }
      TextField::Usize(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          UsizeTextField::Above => self.above = v,
          UsizeTextField::Below => self.below = v,
        }
      }
      TextField::Prefix => {
        if let Value::String(s) = value {
          self.prefix = s.into(); 
        } else {
          return Err(format!("prefix doesnt take {:?}", value))
        }
      }
    }
    Ok(())
  }
}
