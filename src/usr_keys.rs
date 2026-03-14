// src/usr_keys.rs

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


#[derive(Debug)]
enum Key {
  Global, 
  MsgView, 
  LoadUser,
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
impl Key {
  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
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
      key => 
        Err(format!("Keys table does not contain key {}.", key)),
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

impl UserKeys {
  pub fn try_assign(&mut self, key: &Key, value: &Value) 
    -> Result<(), String> 
  {
    let v = Self::try_from_value(value)?;
    match key {
      Key::Global     => self.global = v,
      Key::MsgView    => self.msg_view = v,
      Key::LoadUser   => self.load_usr = v,
      Key::TabView    => self.tab_view = v,
      Key::MoveUp     => self.move_up = v,
      Key::MoveDown   => self.move_down = v,
      Key::MoveLeft   => self.move_left = v,
      Key::MoveRight  => self.move_right = v,
      Key::CycleLeft  => self.cycle_left = v,
      Key::CycleRight => self.cycle_right = v,
      Key::DelTab     => self.delete_tab = v,
      Key::NewTab     => self.new_tab = v,
      Key::Inspect    => self.inspect = v,
      Key::Ack        => self.ack = v,
      Key::Yes        => self.yes = v,
      Key::No         => self.no = v,
      Key::Cancel     => self.cancel = v,
    }
    Ok(())
  }

  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = Key::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }

    Ok(self)
  }


  pub fn try_from_value(value: &Value) 
    -> Result<KeyCode, String> 
  {
    if let Value::String(s) = value {
      if let Some(kc) = Self::keycode_from_string(&s) {
        Ok(kc)
      } else {
        Err("i cant do it".into())
      }
    } else {
      Err("i cant do it".into())
    }
  }

  pub fn keycode_from_string(text: &str) -> Option<KeyCode> {
    match text {
      "esc" | "escape"  => Some(KeyCode::Esc),
      "ent" | "enter"   => Some(KeyCode::Enter),
      "space"           => Some(KeyCode::Char(' ')),
      "left"            => Some(KeyCode::Left),
      "up"              => Some(KeyCode::Up),
      "down"            => Some(KeyCode::Down),
      "right"           => Some(KeyCode::Right),
      t => 
        t
          .chars()
          .next()
          .map(|c| KeyCode::Char(c)),
    }
  }
}
