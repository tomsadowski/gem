// src/usr.rs

use crate::{
  gem::{GemDoc},
  text::{Doc, Text, Editor},
  screen::{Rect, Page},
  usr_layout::{UserLayout},
  usr_keys::{UserKeys},
  dlg::{Dialog, InputType},
};
use crossterm::style::Color;
use toml::{Table, Value};

#[derive(Debug)]
enum Key {
  InitUrl,
  Layout,
  Keys,
}

impl Key {
  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "init_url" => Ok(Self::InitUrl),
      "layout"   => Ok(Self::Layout),
      "keys"     => Ok(Self::Keys),
      key => 
        Err(
          format!(
            "No key named {} in User table", key)),
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

  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = Key::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }

    Ok(self)
  }

  pub fn parse(text: &str) -> Result<Self, String> {
    let table = text.parse::<Table>()
      .map_err(|e| e.to_string())?;
    Self::default().read_table(&table)
  }

  fn try_assign(&mut self, key: &Key, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      Key::InitUrl => {

        if let Value::String(s) = value {
          self.init_url = s.into();

        } else {
          return Err(
            "init_url key expects a string value".into())
        }
      }
      Key::Layout => {

        if let Value::Table(t) = value {
          self.layout = UserLayout::default().read_table(t)?;

        } else {
          return Err(
            "layout key expects a table value".into())
        }
      }
      Key::Keys => {

        if let Value::Table(t) = value {
          self.keys = UserKeys::default().read_table(t)?;

        } else {
          return Err(
            "keys key expects a table value".into())
        }
      }
    }
    Ok(())
  }

  pub fn get_layout(&self, w: u16, h: u16) -> (Page, Page) 
  {
    let rect = Rect::new(w, h);
    self.layout.get_layout(&rect)
  }

  pub fn get_hdr_doc(&self, info: &str, page: &Page) 
    -> Doc 
  {
    let fg = self.layout.banner
      .unwrap_or(Color::White);
    let bg = self.layout.background
      .unwrap_or(Color::Black);
    let line = &String::from("-")
      .repeat(page.text.w);

    Doc::new(
      vec![
        Text::from(info).fg(fg).bg(bg),
        Text::from(line.as_str()).fg(fg).bg(bg), 
      ],
      &page
    )
  }

  pub fn text(&self, page: &Page, text: &str) -> Dialog {

    let mut dlg = Dialog::new(page, text);
    let pos = dlg.input_page.pos();
    let color = self.layout.dialog.unwrap_or(Color::White);
    let editor = Editor::new(&dlg.input_page, "", color);

    dlg.input_type = InputType::Text(editor, pos);
    dlg
  }

  pub fn ack(&self, page: &Page, text: &str) -> Dialog {

    let mut dlg = Dialog::new(page, text);
    dlg.input_type = InputType::Ack(self.keys.ack);
    dlg
  }

  pub fn ask(&self, page: &Page, text: &str) -> Dialog {

    let mut dlg = Dialog::new(page, text);
    dlg.input_type = InputType::Ask
      (self.keys.yes, self.keys.no);
    dlg
  }

  pub fn get_doc(&self, gdoc: &GemDoc, page: &Page) -> Doc {
    let text = self.layout.gemtext_to_text(&gdoc.doc);
    Doc::new(text, &page)
  }

  pub fn get_page(&self, w: u16, h: u16) -> Page {
    let rect = Rect::new(w, h);
    self.layout.get_page_from_rect(&rect)
  }
}
