// src/usr.rs

use crate::util;
use crossterm::{
  style::Color,
  event::KeyCode,
};
use toml::{Table, Value};
use std::str::FromStr;


pub trait UserMod<F>: Sized where F: FromStr<Err = String> {
  fn try_assign(&mut self, field: &F, value: &Value) -> Result<(), String>;

  fn read_table(mut self, table: &Table) -> Result<Self, String> {
    for (key, value) in table.iter() {
      let field = F::from_str(&key)?;
      self.try_assign(&field, value)?;
    }
    Ok(self)
  }
}
#[derive(Clone)]
pub struct User {
  pub init_url: String,
  pub style:    Style,
  pub keys:     Keys,
} 
impl Default for User {
  fn default() -> Self {
    Self {
      init_url:  "src/main.rs".into(),
      style:     Style::default(),
      keys:      Keys::default(),
    }
  }
}
impl UserMod<UserField> for User {
  fn try_assign(&mut self, field: &UserField, value: &Value) 
    -> Result<(), String> 
  {
    match field {
      UserField::InitUrl => {
        if let Value::String(s) = value {
          self.init_url = s.into();
        } else {
          return Err("init_path field expects a string value".into())
        }
      }
      UserField::Style => {
        if let Value::Table(t) = value {
          self.style = Style::default().read_table(t)?;
        } else {
          return Err("style field expects a table value".into())
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
impl FromStr for User {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let table = s.parse::<Table>().map_err(|e| e.to_string())?;
    Self::default().read_table(&table)
  }
}
#[derive(Clone)]
pub struct Style {
  pub x_margin: u16,
  pub y_margin: u16,
  pub banner:   Option<Color>,
  pub bg:       Option<Color>,
  pub fg:       Option<Color>,
} 
impl Default for Style {
  fn default() -> Self {
    Self {
      x_margin: 0,
      y_margin: 0,
      fg:       None,
      bg:       None,
      banner:   None,
    }
  }
}
impl UserMod<StyleField> for Style {
  fn try_assign(&mut self, field: &StyleField, value: &Value) 
    -> Result<(), String> 
  {
    match field {
      StyleField::Color(field) => {
        let v = util::parse_color(value)
          .map_err(|e| format!("{:?} : {}", value, e))?;
        match field {
          ColorField::Fg     => self.fg     = Some(v),
          ColorField::Bg     => self.bg     = Some(v),
          ColorField::Banner => self.banner = Some(v),
        }
      }
      StyleField::Margin(field) => {
        let v = (
          if let Value::Integer(t) = value {
            u16::try_from(*t)
              .map_err(|e| format!("{:?} : {}", value, e))
          } else {
            Err(format!("margin must be a number, not {:?}", value))
          }
        )?;
        match field {
          MarginField::X => self.x_margin = v,
          MarginField::Y => self.y_margin = v,
        }
      }
    }
    Ok(())
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
    let get_keycode = || -> Result<KeyCode, String> {
      if let Value::String(s) = value {
        match s.as_str() {
          "esc" | "escape" => Ok(KeyCode::Esc),
          "ent" | "enter"  => Ok(KeyCode::Enter),
          "space"          => Ok(KeyCode::Char(' ')),
          "left"           => Ok(KeyCode::Left),
          "up"             => Ok(KeyCode::Up),
          "down"           => Ok(KeyCode::Down),
          "right"          => Ok(KeyCode::Right),
          s => 
            s.chars().next().map(|c| KeyCode::Char(c))
            .ok_or("could not parse keycode from string".into()),
        }
      } else {
        Err("could not parse keycode from value".into())
      }
    };
    let value = get_keycode()?;
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

#[derive(Debug)]
enum UserField {
  InitUrl, Style, Keys
}
impl FromStr for UserField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "init_url" => Ok(Self::InitUrl),
      "style"    => Ok(Self::Style),
      "keys"     => Ok(Self::Keys),
      s          => Err(format!("No field {} in User table", s)),
    }
  }
}
#[derive(Debug)]
enum StyleField {
  Color(ColorField), 
  Margin(MarginField),
}
#[derive(Debug)]
enum ColorField {
  Fg, Bg, Banner
}
#[derive(Debug)]
enum MarginField {
  X, Y
}
impl FromStr for StyleField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "fg"       => Ok(Self::Color(ColorField::Fg)),
      "bg"       => Ok(Self::Color(ColorField::Bg)),
      "banner"   => Ok(Self::Color(ColorField::Banner)),
      "x_margin" => Ok(Self::Margin(MarginField::X)),
      "y_margin" => Ok(Self::Margin(MarginField::Y)),
      s => Err(format!("Style table does not contain field {}", s)),
    }
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
impl FromStr for KeysField {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
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
      s => Err(format!("Keys table does not contain field {}", s)),
    }
  }
}
