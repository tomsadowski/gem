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
  usr_layout::{UserLayout},
  usr_keys::{UserKeys},
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
