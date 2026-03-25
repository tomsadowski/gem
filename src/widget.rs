// src/widget.rs

use crate::{
  text::{Page, Tape},
  screen::{Rect, PageView},
};
use crossterm::{
  QueueableCommand,
  style::{
    Color, SetForegroundColor, 
    SetBackgroundColor, Print, ResetColor
  },
  cursor::{self, MoveTo},
};
use std::io::{self, stdout, Write};

// coordinate Page and PageView
pub struct PageWidget {
  pub doc:  Page,
  pub pos:  PageView,
  pub rect: Rect,
  pub text: String,
}
impl PageWidget {
  pub fn new(text: &str, w: u16, h: u16) -> Self {
    let rect = Rect::new(w, h).crop_x(3).crop_y(3);
    let doc = Page::new(text, rect.w);
    let pos = PageView::new(&rect);
    Self {pos, rect, doc, text: text.into()}
  }
  pub fn debug_cursor(&self) -> String {
    format!(
      "data(x: {} y: {}) shift(x: {} y: {}) point(x: {} y: {})",
      self.doc.x(), 
      self.doc.y(), 
      self.pos.x.shift, 
      self.pos.y.shift,
      self.pos.x.point, 
      self.pos.y.point,
      )
  }
  pub fn view<W: Write>(&self, writer: &mut W) -> io::Result<()> {
    writer
      .queue(cursor::Hide)?
      .queue(SetForegroundColor(Color::Rgb {r: 185, g: 180, b: 175 }))?
      .queue(SetBackgroundColor(Color::Rgb {r: 24, g: 24, b: 24}))?;

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
    self.rect = Rect::new(w, h).crop_x(3).crop_y(3);
    self.doc.resize(&self.text, self.rect.w);
    self.pos.resize(&self.doc, &self.rect)
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
