// src/widget.rs

use crate::{
  text::{TextPlane, Linear, Planar},
  screen::{Rect, PlaneView},
};
use crossterm::{
  QueueableCommand,
  style::{
    Print, 
    Color, 
    SetForegroundColor, 
    SetBackgroundColor, 
    ResetColor
  },
  cursor::{self, MoveTo},
};
use std::io::{self, stdout, Write};

// coordinate TextPlane and PlaneView
pub struct TextBox {
  pub fg:   Option<Color>,
  pub bg:   Option<Color>,
  pub doc:  TextPlane,
  pub pos:  PlaneView,
  pub rect: Rect,
  pub text: String,
}
impl TextBox {
  pub fn new(text: &str, rect: &Rect) -> Self {
    let rect = rect.clone();
    let doc = TextPlane::new(text, rect.w);
    let pos = PlaneView::new(&rect);
    Self {
      pos, rect, doc, 
      fg: None,
      bg: None,
      text: text.into()}
  }
  pub fn debug_cursor(&self) -> String {
    format!(
      "data(x: {} y: {}) shift(x: {} y: {}) point(x: {} y: {})",
      self.doc.x_head(), 
      self.doc.y_head(), 
      self.pos.x.shift, 
      self.pos.y.shift,
      self.pos.x.point, 
      self.pos.y.point,
      )
  }
  pub fn view<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if let Some(fg) = self.fg {
      writer.queue(SetForegroundColor(fg))?;
    }
    if let Some(bg) = self.bg {
      writer.queue(SetBackgroundColor(bg))?;
    }
    writer.queue(cursor::Hide)?;

    for (line, y) in self.doc.text[self.pos.y.shift..]
      .iter().zip(self.rect.y..(self.rect.y + self.rect.h)) 
    {
      let mut chars = line.text[self.pos.x.shift..].chars();
      for x in self.rect.x..(self.rect.x + self.rect.w) {
        writer
          .queue(MoveTo(x, y))?
          .queue(Print(chars.next().unwrap_or(' ')))?;
      }
    }
    let debug = self.debug_cursor();
    let mut chars = debug.chars();
    for x in self.rect.x..(self.rect.x + self.rect.w) {
      writer
        .queue(MoveTo(x, self.rect.y + self.rect.h + 1))?
        .queue(Print(chars.next().unwrap_or(' ')))?;
    }
    writer
      .queue(ResetColor)?
      .queue(MoveTo(self.pos.x.point, self.pos.y.point))?
      .queue(cursor::Show)?;
    Ok(())
  }
  pub fn resize(&mut self, w: u16, h: u16) {
    self.rect = Rect::new(w, h);
    self.rect.crop_x(3);
    self.rect.crop_y(3);
    self.doc.resize(&self.text, self.rect.w);
    self.pos.resize(&self.doc, &self.rect)
  }
  pub fn delete(&mut self) -> bool {
    if self.doc.delete() {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.doc.backspace() {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.doc.insert(c) {
      self.pos.update(&self.doc, &self.rect);
      true 
    } else {false}
  }
  pub fn left(&mut self) -> bool {
    if self.doc.left(1) == 0 {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn right(&mut self) -> bool {
    if self.doc.right(1) == 0 {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn down(&mut self) -> bool {
    if self.doc.down(1) {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn up(&mut self) -> bool {
    if self.doc.up(1) {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
}
