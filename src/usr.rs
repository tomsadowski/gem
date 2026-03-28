// src/usr.rs

use crate::util;
use crossterm::{
  style::Color,
  event::KeyCode,
};
use toml::{Table, Value};


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
  Keys,
}
impl Field for UserField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "init_url" => Ok(Self::InitPath),
      "layout"   => Ok(Self::Layout),
      "keys"     => Ok(Self::Keys),
      s => 
        Err(format!("No field {} in User table", s)),
    }
  }
}

#[derive(Clone)]
pub struct User {
  pub init_path: String,
  pub layout:    Layout,
  pub keys:      Keys,
} 
impl Default for User {
  fn default() -> Self {
    Self {
      init_path: ".".into(),
      layout:    Layout::default(),
      keys:      Keys::default(),
    }
  }
}
impl User {
  pub fn try_from_string(s: &str) -> Result<Self, String> {
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
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
          self.layout = Layout::default().read_table(t)?;
        } else {
          return Err("layout field expects a table value".into())
        }
      }
      UserField::Keys => {
        if let Value::Table(t) = value {
          self.keys = Keys::default().read_table(t)?;
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
        Err(format!("No field {} in Keys table", s)),
    }
  }
}

#[derive(Clone)]
pub struct Keys {
  pub global:      KeyCode,
  pub cancel:      KeyCode,
  pub load_usr:    KeyCode,
  pub msg_view:    KeyCode,
  pub tab_view:    KeyCode,
  pub up:          KeyCode,
  pub down:        KeyCode,
  pub left:        KeyCode,
  pub right:       KeyCode,
  pub cycle_left:  KeyCode,
  pub cycle_right: KeyCode,
  pub inspect:     KeyCode,
  pub delete_tab:  KeyCode,
  pub new_tab:     KeyCode,
  pub ack:         KeyCode, 
  pub yes:         KeyCode, 
  pub no:          KeyCode
} 
impl Default for Keys {
  fn default() -> Self {
    Self {
      global:      KeyCode::Char('g'),
      cancel:      KeyCode::Esc,
      load_usr:    KeyCode::Char('c'),
      msg_view:    KeyCode::Char('m'),
      tab_view:    KeyCode::Char('t'),
      up:          KeyCode::Up,
      down:        KeyCode::Down,
      left:        KeyCode::Left,
      right:       KeyCode::Right,
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
impl UserMod<KeysField> for Keys {
  fn try_assign(&mut self, field: &KeysField, value: &Value) 
    -> Result<(), String> 
  {
    let value = Self::try_from_value(value)?;
    match field {
      KeysField::Global     => self.global      = value,
      KeysField::MsgView    => self.msg_view    = value,
      KeysField::LoadUser   => self.load_usr    = value,
      KeysField::TabView    => self.tab_view    = value,
      KeysField::MoveUp     => self.up          = value,
      KeysField::MoveDown   => self.down        = value,
      KeysField::MoveLeft   => self.left        = value,
      KeysField::MoveRight  => self.right       = value,
      KeysField::CycleLeft  => self.cycle_left  = value,
      KeysField::CycleRight => self.cycle_right = value,
      KeysField::DelTab     => self.delete_tab  = value,
      KeysField::NewTab     => self.new_tab     = value,
      KeysField::Inspect    => self.inspect     = value,
      KeysField::Ack        => self.ack         = value,
      KeysField::Yes        => self.yes         = value,
      KeysField::No         => self.no          = value,
      KeysField::Cancel     => self.cancel      = value,
    }
    Ok(())
  }
}
impl Keys {
  pub fn try_from_value(value: &Value) -> Result<KeyCode, String> {
    if let Value::String(s) = value {
      match s.as_str() {
        "esc" | "escape"  => Ok(KeyCode::Esc),
        "ent" | "enter"   => Ok(KeyCode::Enter),
        "space"           => Ok(KeyCode::Char(' ')),
        "left"            => Ok(KeyCode::Left),
        "up"              => Ok(KeyCode::Up),
        "down"            => Ok(KeyCode::Down),
        "right"           => Ok(KeyCode::Right),
        s => 
          s.chars().next().map(|c| KeyCode::Char(c))
          .ok_or("could not parse keycode from string".into()),
      }
    } else {
      Err("could not parse keycode from value".into())
    }
  }
}

#[derive(Debug)]
enum LayoutField {
  Color(ColorField), 
  Dim(DimField),
}
impl Field for LayoutField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "x_text" => Ok(Self::Dim(DimField::XText)),
      "y_text" => Ok(Self::Dim(DimField::YText)),
      "x_page" => Ok(Self::Dim(DimField::XPage)),
      "y_page" => Ok(Self::Dim(DimField::YPage)),
      "banner" => Ok(Self::Color(ColorField::Banner)),
      "text"   => Ok(Self::Color(ColorField::Text)),
      "background" | "bg" 
               => Ok(Self::Color(ColorField::Bg)),
      s => 
        Err(format!("Layout table does not contain field {}.", s)),
    }
  }
}

#[derive(Debug)]
enum ColorField {
  Text, Bg, Banner,
}
impl ColorField {
  pub fn try_parse_value(&self, value: &Value) -> Result<Color, String> {
    util::parse_color(value).map_err(|e| format!("{:?} : {}", self, e))
  }
}

#[derive(Debug)]
enum DimField {
  XPage, YPage, XText, YText
}
impl DimField {
  pub fn try_parse_value(&self, value: &Value) -> Result<u16, String> {
    if let Value::Integer(t) = value {
      u16::try_from(*t).map_err(|e| format!("{:?} : {}", self, e))
    } else {
      Err(format!("prefix doesnt take {:?}", value))
    }
  }
}

#[derive(Clone)]
pub struct Layout {
  pub x_text:     u16,
  pub y_text:     u16,
  pub x_page:     u16,
  pub y_page:     u16,
  pub background: Option<Color>,
  pub banner:     Option<Color>,
  pub text:       Option<Color>,
} 
impl Default for Layout {
  fn default() -> Self {
    Self {
      x_text:     0,
      y_text:     0,
      x_page:     0,
      y_page:     0,
      background: None,
      banner:     None,
      text:       None,
    }
  }
}
impl UserMod<LayoutField> for Layout {
  fn try_assign(&mut self, field: &LayoutField, value: &Value) 
    -> Result<(), String> 
  {
    match field {
      LayoutField::Color(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          ColorField::Bg     => self.background = Some(v),
          ColorField::Banner => self.banner     = Some(v),
          ColorField::Text   => self.text       = Some(v),
        }
      }
      LayoutField::Dim(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          DimField::XText => self.x_text = v,
          DimField::YText => self.y_text = v,
          DimField::XPage => self.x_page = v,
          DimField::YPage => self.y_page = v,
        }
      }
    }
    Ok(())
  }
}
