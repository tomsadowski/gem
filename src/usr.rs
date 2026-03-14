// src/usr.rs

use crate::{
  gem::{
    GemDoc
  },
  text::{
    Doc,
  },
  screen::{
    Rect, Frame,
  },
  usr_layout::{UserLayout},
  usr_keys::{UserKeys},
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

  pub fn parse(text: &str) -> Result<Self, String> {

    let mut usr = Self::default();
    let table = text.parse::<Table>()
      .unwrap_or_default();

    if let Some(Value::String(s)) = table.get("init_url") {
      usr.init_url = s.into();
    }

    if let Some(Value::Table(t)) = table.get("layout") {
      usr.layout = UserLayout::default().read_table(t)?;
    }

    if let Some(Value::Table(t)) = table.get("keys") {
      usr.keys = UserKeys::default().read_table(t)?;
    }
    
    Ok(usr)
  }
}
