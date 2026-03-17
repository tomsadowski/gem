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
use std::{
  str::Chars,
  iter::{Enumerate, Skip},
};


pub trait TextWidget {

  fn x_scroll(&self) -> usize;

  fn y_scroll(&self) -> usize;

  fn x_cursor(&self) -> u16;

  fn y_cursor(&self) -> u16;

  fn y_len(&self) -> usize;

  fn lines(&self) -> Vec<(&Color, &Color, &String)>;

  fn fg(&self, idx: usize) -> &Color;

  fn bg(&self, idx: usize) -> &Color;

  fn get_lines(&self, page: &Page) 
    -> Vec<(u16, &Color, &Color, &String)> 
  {
    let line_start = 
      self.y_len()
      .saturating_sub(1)
      .min(self.y_scroll());

    let line_end = 
      line_start
      .saturating_add(page.text.h)
      .min(self.y_len());

    self.lines()[line_start..line_end]
      .iter()
      .enumerate() 
      .map(
        |(scr_idx, (fg, bg, line))| {
          let scr_idx = u16_or_0(scr_idx);
          (scr_idx, *fg, *bg, *line)
        })
      .collect()
  }


  fn get_x_iter<'a>(&self, line: &'a String) 
    -> Skip<Chars<'a>>
  {
      line
        .chars()
        .skip(self
          .x_scroll()
          .min(line
            .len()
            .saturating_sub(1)))
  }


  fn view<W>(&self, page: &Page, wrt: &mut W) 
    -> io::Result<()> 
  where W: Write,
  {
    for (scr_idx, fg, bg, line) in 
      self.get_lines(page).iter()
    {
      wrt
        .queue(SetForegroundColor(**fg))?
        .queue(SetBackgroundColor(**bg))?;

      let mut chars = self.get_x_iter(line);

      let Range16 {start: x_start, end: x_end} = 
        page.text.x();

      for x_pos in x_start..x_end {
        wrt
          .queue(MoveTo(x_pos, scr_idx + page.text.y))?
          .queue(Print(chars.next().unwrap_or(' ')))?;
      }
    }

    wrt
      .queue(MoveTo(self.x_cursor(), self.y_cursor()))?;

    Ok(())
  }
}


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
impl TextWidget for Doc {

  fn x_cursor(&self) -> u16 {
    self.pos.x.cursor
  }

  fn y_cursor(&self) -> u16 {
    self.pos.y.cursor
  }

  fn x_scroll(&self) -> usize {
    self.pos.x.scroll
  }

  fn y_scroll(&self) -> usize {
    self.pos.y.scroll
  }

  fn y_len(&self) -> usize {
    self.lines.len()
  }

  fn lines(&self) -> Vec<(&Color, &Color, &String)> {
    self.lines
      .iter()
      .map(|(u, s)| {
          let fg = &self.text[*u].fg;
          let bg = &self.text[*u].bg;
          (fg, bg, s)
        })
      .collect()
  }

  fn fg(&self, idx: usize) -> &Color {
    &self.text[idx].fg
  }

  fn bg(&self, idx: usize) -> &Color {
    &self.text[idx].bg
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
  pub pos:   Pos,
  pub text:  String,
  pub fg:    Color,
  pub bg:    Color,
}
impl TextWidget for Editor {

  fn x_cursor(&self) -> u16 {
    self.pos.x.cursor
  }

  fn y_cursor(&self) -> u16 {
    self.pos.y.cursor
  }


  fn x_scroll(&self) -> usize {
    self.pos.x.scroll
  }

  fn y_scroll(&self) -> usize {
    self.pos.y.scroll
  }

  fn y_len(&self) -> usize {
    self.text.len()
  }

  fn lines(&self) -> Vec<(&Color, &Color, &String)> {
    vec![(&self.fg, &self.bg, &self.text)]
  }

  fn fg(&self, _: usize) -> &Color {
    &self.fg
  }

  fn bg(&self, _: usize) -> &Color {
    &self.bg
  }
}
impl Editor {

  fn wrap_line(txt: &str) -> Vec<(usize, String)> {
    txt
      .lines()
      .map(|l| (usize::MIN, String::from(l)))
      .collect()
  }


  pub fn new(page: &Page) -> Self {

    Self {
      pos:    page.pos(),
      fg:     Color::White,
      bg:     Color::Black,
      text:   "".into(),
    }
  }


  pub fn move_left(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    self.pos.move_left(&page, step)
  }


  pub fn move_right(&mut self, page: &Page, step: u16) 
    -> bool 
  {
    self.pos.move_right(&page, self.text.len(), step)
  }


  pub fn delete(&mut self, page: &Page) -> bool {
    let text = page.text.x();
    let idx = self.pos.x.data_idx(&text);

    if idx >= self.text.len() || self.text.len() == 0 {
      return false
    }
    self.text.remove(idx);
    if self.pos.x.cursor + 1 != text.end {
      self.pos.move_right(&page, self.text.len(), 1)
    } else {
      false
    }
  }


  pub fn backspace(&mut self, page: &Page) -> bool {
    if self.text.len() == 0 {
      return false
    }
    let idx = self.pos.x.data_idx(&page.text.x());

    self.pos.move_left(&page, 1);
    self.text.remove(idx);

    if self.pos.x.cursor + 1 != page.text.x().end {
      self.pos.move_right(&page, self.text.len(), 1);
    }
    true
  }


  pub fn insert(&mut self, page: &Page, c: char) -> bool {
    let idx = self.pos.x.data_idx(&page.text.x()) + 1;
    if idx >= self.text.len() || self.text.len() == 0 {
      self.text.push(c);
    } else {
      self.text.insert(idx, c);
    }
    self.pos.move_right(&page, self.text.len(), 1);
    true
  }
}

