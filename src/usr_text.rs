// src/usr_text.rs

use crate::{
  text::{Text},
  util::{parse_color},
};
use crossterm::{
  style::{Color},
};
use toml::{
  Table, Value,
};

#[derive(Debug)]
enum Key {
  Color(ColorKey), 
  Usize(UsizeKey), 
  Prefix,
}
impl Key {
  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "fg"      => Ok(Self::Color(ColorKey::Fg)),
      "bg"      => Ok(Self::Color(ColorKey::Bg)),
      "above"   => Ok(Self::Usize(UsizeKey::Above)),
      "below"   => Ok(Self::Usize(UsizeKey::Below)),
      "prefix"  => Ok(Self::Prefix),
      key => 
        Err(format!("{} no such field in the table", key)),
    }
  }
}

#[derive(Debug)]
enum ColorKey {
  Fg, Bg,
}
impl ColorKey {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<Color, String>
  {
    parse_color(value)
      .map_err(|e| format!("{:?} : {}", self, e))
  }
}

#[derive(Debug)]
enum UsizeKey {
  Above, Below,
}
impl UsizeKey {
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

  fn try_assign(&mut self, key: &Key, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      Key::Color(k) => {
        let v = k.try_parse_value(&value)?;
        match k {
          ColorKey::Fg => self.fg = Some(v),
          ColorKey::Bg => self.bg = Some(v),
        }
        Ok(())
      }
      Key::Usize(k) => {
        let v = k.try_parse_value(&value)?;
        match k {
          UsizeKey::Above => self.above = v,
          UsizeKey::Below => self.below = v,
        }
        Ok(())
      }
      Key::Prefix => {
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
      let k = Key::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }

    Ok(self)
  }
}
