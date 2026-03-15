// src/text.rs

use crate::{
  page::{Page, Range16},
  pos::{Pos, PosCol},
  util::{wrap, u16_or_0},
};
use crossterm::{
  QueueableCommand,
  style::{
    Color, SetForegroundColor, 
    SetBackgroundColor, Print, ResetColor},
  cursor::MoveTo,
};
use std::io::{self, Write};


#[derive(Clone, Debug)]
pub struct Text {
  pub text:   String,
  pub fg:     Color,
  pub bg:     Color,
  pub above:  usize,
  pub below:  usize,
  pub prefix: String,
  pub wrap:   bool,
}
impl From<&str> for Text {
  fn from(item: &str) -> Self {
    Self {
      above:  0,
      below:  0,
      text:   item.into(), 
      fg:     Color::White, 
      bg:     Color::Black,
      prefix: "".into(),
      wrap:   false,
    }
  }
}
impl Default for Text {
  fn default() -> Self {
    Self::from("")
  }
}
impl Text {
  pub fn write_page<W>(&self, 
                        page: &Page, 
                        wrt: &mut W) 
    -> io::Result<()>
  where W: Write
  {
    wrt
      .queue(SetForegroundColor(self.fg))?;

    let mut chars = self.text.chars();

    let Range16 {start: x_start, end: x_end} = 
      page.text.x();

    for x_pos in x_start..x_end {
      wrt
        .queue(MoveTo(x_pos, page.text.y))?
        .queue(Print(chars.next().unwrap_or(' ')))?;
    }

    Ok(())
  }

  pub fn prefix(mut self, s: &str) -> Self {
    self.prefix = String::from(s);
    self.text.insert_str(0, s);
    self
  }

  pub fn above(mut self, u: usize) -> Self {
    self.above = u;
    self
  }

  pub fn below(mut self, u: usize) -> Self {
    self.below = u;
    self
  }

  pub fn fg(mut self, col: Color) -> Self {
    self.fg = col;
    self
  }

  pub fn bg(mut self, col: Color) -> Self {
    self.bg = col;
    self
  }

  pub fn wrap(mut self) -> Self {
    self.wrap = true;
    self
  }
}

pub struct Doc {
  pub pos:    Pos,
  pub text:   Vec<Text>,
  pub lines:  Vec<(usize, String)>,
} 
impl Default for Doc {
  fn default() -> Self {
    Self {
      pos:    Pos::default(), 
      text:   vec![], 
      lines:  vec![]
    }
  }
}
impl Doc {

  pub fn text(mut self, text: Vec<Text>) -> Self {
    self.text = text;
    self
  }

  pub fn new(text: Vec<Text>, page: &Page) -> Self {

    let lines = Self::wrap_list(&text, page.text.w);
    let pos = page.pos();

    Self {pos, lines, text}
  }

  pub fn resize(&mut self, page: &Page) {
    self.lines = 
      Self::wrap_list(&self.text, page.text.w);
  }

  pub fn select(&self, page: &Page) -> Option<usize> {

    let line_idx = self.pos.y
      .data_idx(&page.text.y());

    self.lines
      .get(line_idx)
      .map(|(text_idx, _)| *text_idx)
  }

  fn y(&self) -> usize {
    self.lines.len()
  }

  fn x(&self, y: usize) -> Option<usize> {
    self.lines
      .get(y)
      .map(|(_, lines)| lines.len())
  }

  pub fn move_left(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    self.pos.x.move_backward(&page.x(), step)
  }

  pub fn move_right(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    self
      .x(self.pos.y.data_idx(&page.text.y()))
      .map(|x| 
        self.pos.x.move_forward(&page.x(), x, step))
      .unwrap_or(false)
  }

  pub fn move_up(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    if self.pos.y.move_backward(&page.y(), step) {
      self.move_into_x(page); true
    } else {false}
  }

  pub fn move_down(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    if self.pos.y
      .move_forward(&page.y(), self.y(), step) 
    {
      self.move_into_x(page); 
      true
    } else {
      false
    }
  }

  pub fn move_into_x(&mut self, page: &Page) {
    let idx = self.pos.y
      .data_idx(&page.text.y())
      .min(self.y().saturating_sub(1));
    self
      .x(idx)
      .inspect(|d| self.pos.x.move_into(&page.x(), *d));
  }

  pub fn view<W>(&self, page: &Page, wrt: &mut W) 
    -> io::Result<()> 
  where W: Write
  {
    let line_start = self.lines.len()
      .saturating_sub(1)
      .min(self.pos.y.scroll);

    let line_end = line_start
      .saturating_add(page.text.h)
      .min(self.lines.len());

    for (scr_idx, (text_idx, line)) in 
      self.lines[line_start..line_end]
        .iter()
        .enumerate() 
    {
      wrt
        .queue(
          SetForegroundColor(
            self.text[*text_idx].fg))?
        .queue(
          SetBackgroundColor(
            self.text[*text_idx].bg))?;

      let mut chars = line
        .chars()
        .skip(line.len()
          .saturating_sub(1)
          .min(self.pos.x.scroll));

      let Range16 {start: x_start, end: x_end} = 
        page.text.x();

      for x_pos in x_start..x_end {
        wrt
          .queue(
            MoveTo(x_pos, 
              u16_or_0(scr_idx) + page.text.y))?
          .queue(
            Print(chars.next().unwrap_or(' ')))?;
      }
    }

    wrt
        .queue(
          MoveTo(
            self.pos.x.cursor, 
            self.pos.y.cursor))?
        .queue(ResetColor)?;

    Ok(())
  }

  fn wrap_list(lines: &Vec<Text>, w: usize) 
    -> Vec<(usize, String)> 
  {
    let mut display: Vec<(usize, String)> = vec![];

    for (i, l) in lines.iter().enumerate() {

      for x in 0..l.above {
        display.push((i, "".to_string()));
      }

      let v = 
        if l.wrap {
          wrap(&l.text, w)
        } else {
          vec![l.text.clone()]
        };

      for s in v.iter() {
        display.push((i, s.to_string()));
      }

      for x in 0..l.below {
        display.push((i, "".to_string()));
      }
    }

    display
  }
} 

#[derive(Clone)]
pub struct Editor {
  pub pos:    PosCol,
  pub txt:    String,
  pub color:  Color,
}
impl Editor {
  pub fn new(page: &Page, txt: &str, color: Color) 
    -> Self 
  {
    Self {
      pos:    page.pos().x,
      color:  color,
      txt:    txt.into(),
    }
  }

  pub fn move_left(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    self.pos.move_backward(&page.x(), step)
  }

  pub fn move_right(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    self.pos.move_forward(&page.x(), self.txt.len(), step)
  }

  pub fn write_page<W>(&self, page: &Page, wrt: &mut W) 
    -> io::Result<()> 
  where W: Write
  {
    wrt
      .queue(SetForegroundColor(self.color))?;

    let mut chars = self.txt
      .chars()
      .skip(self.txt.len()
          .saturating_sub(1)
          .min(self.pos.scroll));

    let Range16 {start: x_start, end: x_end} = 
      page.text.x();

    for x_pos in x_start..x_end {
      let c = chars.next().unwrap_or(' ');

      wrt
        .queue(MoveTo(x_pos, self.pos.cursor))?
        .queue(Print(c))?;
    }
    Ok(())
  }

  pub fn delete(&mut self, page: &Page, pos: &mut Pos) 
    -> bool 
  {
    let text = page.text.x();
    let idx = pos.x.data_idx(&text);

    if idx >= self.txt.len() || self.txt.len() == 0 {
      return false
    }

    self.txt.remove(idx);
    if pos.x.cursor + 1 != text.end {
      pos.x.move_forward(&page.x(), self.txt.len(), 1)
    } else {
      false
    }
  }

  pub fn backspace(&mut self, page: &Page, pos: &mut Pos) 
    -> bool 
  {
    if self.txt.len() == 0 {
      return false
    }

    let idx = pos.x.data_idx(&page.text.x());

    pos.x.move_backward(&page.x(), 1);
    self.txt.remove(idx);

    if pos.x.cursor + 1 != page.text.x().end {
      pos.x.move_forward(&page.x(), self.txt.len(), 1);
    }

    true
  }

  pub fn insert(&mut self, 
                page: &Page, 
                pos: &mut Pos, 
                c: char) 
    -> bool 
  {
    let idx = pos.x.data_idx(&page.text.x()) + 1;
    if idx >= self.txt.len() || self.txt.len() == 0 {
      self.txt.push(c);
    } else {
      self.txt.insert(idx, c);
    }
    pos.x.move_forward(&page.x(), self.txt.len(), 1);
    true
  }
}
