// src/widget.rs

use crate::{
  text::{TextPlane, Linear, Planar},
  screen::{Rect, PlaneView},
};
use crossterm::{
  QueueableCommand, cursor,
  style::{Print, Color, SetForegroundColor, SetBackgroundColor, ResetColor},
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
    let doc  = TextPlane::new(text, rect.w);
    let pos  = PlaneView::new(&rect);
    Self {
      fg:   None,
      bg:   None,
      text: text.into(),
      pos, rect, doc, 
    }
  }
  pub fn view<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    if let Some(fg) = self.fg {
      writer.queue(SetForegroundColor(fg))?;
    }
    if let Some(bg) = self.bg {
      writer.queue(SetBackgroundColor(bg))?;
    }
    writer.queue(cursor::Hide)?;

    for (line, y) in 
      self.doc.text[self.pos.y_scroll()..]
        .iter().zip(self.rect.y_range()) 
    {
      let mut chars = line.text[self.pos.x_scroll()..].chars();
      for x in self.rect.x_range() {
        writer
          .queue(cursor::MoveTo(x, y))?
          .queue(Print(chars.next().unwrap_or(' ')))?;
      }
    }
    self.debug_cursor(writer)?;
    writer
      .queue(ResetColor)?
      .queue(cursor::MoveTo(self.pos.x_cursor(), self.pos.y_cursor()))?
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
  pub fn left(&mut self, step: usize) -> bool {
    if self.doc.left(step) == 0 {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn right(&mut self, step: usize) -> bool {
    if self.doc.right(step) == 0 {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn down(&mut self, step: usize) -> bool {
    if self.doc.down(step) {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  pub fn up(&mut self, step: usize) -> bool {
    if self.doc.up(step) {
      self.pos.update(&self.doc, &self.rect);
      true
    } else {false}
  }
  fn debug_cursor<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    let debug = format!(
      "data(x: {} y: {}) scroll(x: {} y: {}) cursor(x: {} y: {})",
      self.doc.x_head(), 
      self.doc.y_head(), 
      self.pos.x_scroll(), 
      self.pos.y_scroll(),
      self.pos.x_cursor(), 
      self.pos.y_cursor(),
      );
    let mut chars = debug.chars();
    for x in self.rect.x_range() {
      writer
        .queue(cursor::MoveTo(x, self.rect.y_end() + 1))?
        .queue(Print(chars.next().unwrap_or(' ')))?;
    }
    Ok(())
  }
}
