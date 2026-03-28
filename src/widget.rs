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
  pub page_rect: Rect,
  pub text_rect: Rect,
  pub text: String,
}
impl TextBox {
  pub fn new(text: &str, rect: &Rect) -> Self {
    let mut text_rect = rect.clone();
    text_rect.crop_x(1);
    text_rect.crop_y(1);
    let doc  = TextPlane::new(text, text_rect.w);
    let pos  = PlaneView::new(&text_rect);
    Self {
      fg:   None,
      bg:   None,
      text: text.into(),
      page_rect: rect.clone(),
      pos, text_rect, doc, 
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

    for x in self.page_rect.x_range() {
      for y in self.page_rect.north_range(&self.text_rect) {
        writer.queue(cursor::MoveTo(x, y))?.queue(Print(' '))?;
      }
      for y in self.page_rect.south_range(&self.text_rect) {
        writer.queue(cursor::MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    for y in self.text_rect.y_range() {
      for x in self.page_rect.east_range(&self.text_rect) {
        writer.queue(cursor::MoveTo(x, y))?.queue(Print(' '))?;
      }
      for x in self.page_rect.west_range(&self.text_rect) {
        writer.queue(cursor::MoveTo(x, y))?.queue(Print(' '))?;
      }
    }
    let mut lines = self.doc.text[self.pos.y_scroll()..].iter();
    for y in self.text_rect.y_range() {
      if let Some(line) = lines.next() {
        let mut chars = line.text[self.pos.x_scroll()..].chars();
        for x in self.text_rect.x_range() {
          writer.queue(cursor::MoveTo(x, y))?;
          if let Some(c) = chars.next() {
            writer.queue(Print(c))?;
          } else {
            writer.queue(Print(' '))?;
          }
        }
      } else {
        for x in self.text_rect.x_range() {
          writer.queue(cursor::MoveTo(x, y))?.queue(Print(' '))?;
        }
      }
    }
    writer
      .queue(ResetColor)?
      .queue(cursor::MoveTo(self.pos.x_cursor(), self.pos.y_cursor()))?
      .queue(cursor::Show)?;
    Ok(())
  }
  pub fn resize(&mut self, rect: &Rect) {
    self.page_rect = rect.clone();
    self.text_rect = rect.clone();
    self.text_rect.crop_x(1);
    self.text_rect.crop_y(1);
    self.doc.resize(&self.text, self.text_rect.w);
    self.pos.resize(&self.doc, &self.text_rect);
  }
  pub fn delete(&mut self) -> bool {
    if self.doc.delete() {
      self.pos.update(&self.doc, &self.text_rect);
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.doc.backspace() {
      self.pos.update(&self.doc, &self.text_rect);
      true
    } else {false}
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.doc.insert(c) {
      self.pos.update(&self.doc, &self.text_rect);
      true 
    } else {false}
  }
  pub fn left(&mut self, step: usize) -> bool {
    if self.doc.left(step) == 0 {
      self.pos.update(&self.doc, &self.text_rect);
      true
    } else {false}
  }
  pub fn right(&mut self, step: usize) -> bool {
    if self.doc.right(step) == 0 {
      self.pos.update(&self.doc, &self.text_rect);
      true
    } else {false}
  }
  pub fn down(&mut self, step: usize) -> bool {
    if self.doc.down(step) {
      self.pos.update(&self.doc, &self.text_rect);
      true
    } else {false}
  }
  pub fn up(&mut self, step: usize) -> bool {
    if self.doc.up(step) {
      self.pos.update(&self.doc, &self.text_rect);
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
    for x in self.text_rect.x_range() {
      writer
        .queue(cursor::MoveTo(x, self.page_rect.y_end()))?
        .queue(Print(chars.next().unwrap_or(' ')))?;
    }
    Ok(())
  }
}
