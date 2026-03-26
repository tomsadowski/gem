// src/usr.rs

use crate::{
  text::{TextPlane, Planar, Linear},
  screen::{Rect, PlaneView},
  widget::{TextBox},
  util,
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
  pub keys:      UserKeys,
} 
impl Default for User {
  fn default() -> Self {
    Self {
      init_path: ".".into(),
      layout:    Layout::default(),
      keys:      UserKeys::default(),
    }
  }
}
impl Field for User {
  fn try_from_string(s: &str) -> Result<Self, String> {
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
enum UserKeysField {
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
impl Field for UserKeysField {
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
impl UserMod<UserKeysField> for UserKeys {
  fn try_assign(&mut self, field: &UserKeysField, value: &Value) 
    -> Result<(), String> 
  {
    let value = Self::try_from_value(value)?;
    match field {
      UserKeysField::Global     => self.global      = value,
      UserKeysField::MsgView    => self.msg_view    = value,
      UserKeysField::LoadUser   => self.load_usr    = value,
      UserKeysField::TabView    => self.tab_view    = value,
      UserKeysField::MoveUp     => self.move_up     = value,
      UserKeysField::MoveDown   => self.move_down   = value,
      UserKeysField::MoveLeft   => self.move_left   = value,
      UserKeysField::MoveRight  => self.move_right  = value,
      UserKeysField::CycleLeft  => self.cycle_left  = value,
      UserKeysField::CycleRight => self.cycle_right = value,
      UserKeysField::DelTab     => self.delete_tab  = value,
      UserKeysField::NewTab     => self.new_tab     = value,
      UserKeysField::Inspect    => self.inspect     = value,
      UserKeysField::Ack        => self.ack         = value,
      UserKeysField::Yes        => self.yes         = value,
      UserKeysField::No         => self.no          = value,
      UserKeysField::Cancel     => self.cancel      = value,
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
  Color(LayoutColorField), 
  U16(LayoutDimField),
}
impl Field for LayoutField {
  fn try_from_string(s: &str) -> Result<Self, String> {
    match s {
      "x_text" => Ok(Self::U16(LayoutDimField::XText)),
      "y_text" => Ok(Self::U16(LayoutDimField::YText)),
      "x_page" => Ok(Self::U16(LayoutDimField::XPage)),
      "y_page" => Ok(Self::U16(LayoutDimField::YPage)),
      "banner" => Ok(Self::Color(LayoutColorField::Banner)),
      "text"   => Ok(Self::Color(LayoutColorField::Text)),
      "background" | "bg" 
               => Ok(Self::Color(LayoutColorField::Bg)),
      s => 
        Err(format!("Layout table does not contain field {}.", s)),
    }
  }
}

#[derive(Debug)]
enum LayoutColorField {
  Text, Bg, Banner, Border, Dlg,
}
impl LayoutColorField {
  pub fn try_parse_value(&self, value: &Value) -> Result<Color, String> {
    util::parse_color(value).map_err(|e| format!("{:?} : {}", self, e))
  }
}

#[derive(Debug)]
enum LayoutDimField {
  XPage, YPage, XText, YText
}
impl LayoutDimField {
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
  pub border:     Option<Color>,
  pub dialog:     Option<Color>,
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
      border:     None,
      dialog:     None,
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
          LayoutColorField::Bg     => self.background = Some(v),
          LayoutColorField::Banner => self.banner     = Some(v),
          LayoutColorField::Border => self.border     = Some(v),
          LayoutColorField::Dlg    => self.dialog     = Some(v),
          LayoutColorField::Text   => self.text       = Some(v),
        }
      }
      LayoutField::U16(field) => {
        let v = field.try_parse_value(&value)?;
        match field {
          LayoutDimField::XText => self.x_text = v,
          LayoutDimField::YText => self.y_text = v,
          LayoutDimField::XPage => self.x_page = v,
          LayoutDimField::YPage => self.y_page = v,
        }
      }
    }
    Ok(())
  }
}
impl Layout {
  pub fn get_rect_from_dim(&self, w: u16, h: u16) -> Rect {
    Rect::new(w, h).crop_x(self.x_page).crop_y(self.y_page)
  }
}
