// src/screen.rs

use crate::{
  util::{u16_or_0},
  pos::{Pos},
};
use std::cmp::min;

pub trait Dim {
  fn w(&self) -> usize;
  fn h(&self) -> usize;
}

#[derive(Clone)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: usize,
  pub h: usize,
}

impl Dim for Rect {
  fn w(&self) -> usize {
    self.w
  }
  fn h(&self) -> usize {
    self.h
  }
}

impl Rect {
  pub fn new(w: u16, h: u16) -> Self {
    let w = usize::from(w);
    let h = usize::from(h);

    Self {x: 0, y: 0, w, h}
  }

  pub fn pos(&self) -> Pos {
    Pos::from(self)
  }

  pub fn row(&self, r: u16) -> Self {
    Self {
      x: self.x, 
      y: self.y + r,
      h: 1,
      w: self.w
    }
  }

  pub fn x(&self) -> Range16 {
    Range16 {
      start: self.x, 
      end:   self.x + u16_or_0(self.w)
    }
  }

  pub fn y(&self) -> Range16 {
    Range16 {
      start: self.y, 
      end:   self.y + u16_or_0(self.h)
    }
  }

  pub fn resize(&mut self, w: usize, h: usize) {

    self.w = w; 
    self.h = h;
  }

  pub fn crop_y(&self, step: u16) -> Self {

    let rect = self.clone();
    rect.crop_north(step).crop_south(step)
  }

  pub fn crop_x(&self, step: u16) -> Self {

    let rect = self.clone();
    rect.crop_east(step).crop_west(step)
  }

  pub fn crop_south(&self, step: u16) -> Self {

    let mut rect = self.clone();

    if usize::from(step) < rect.h {
      rect.h -= usize::from(step);
    }
    rect
  }

  pub fn crop_east(&self, step: u16) -> Self {

    let mut rect = self.clone();

    if usize::from(step) < rect.w {
      rect.w -= usize::from(step)
    }
    rect
  }

  pub fn crop_north(&self, step: u16) -> Self {

    let mut rect = self.clone();

    if usize::from(step) * 2 < rect.h {
      rect.y += step;
      rect.h -= usize::from(step);
    }
    rect
  }

  pub fn crop_west(&self, step: u16) -> Self {

    let mut rect = self.clone();

    if usize::from(step) * 2 < rect.w {
      rect.x += step;
      rect.w -= usize::from(step);
    }
    rect
  }
}

#[derive(Clone, Debug)]
pub struct Range16 {
  pub start:  u16, 
  pub end:    u16
}

impl Range16 {
  // if for some reason a > b, just swap them
  pub fn new(start: u16, end: u16) -> Range16 {
    if start > end {
      Range16 {end, start}

    } else {
      Range16 {start, end}
    }
  }

  pub fn get_data_end(&self, dlen: usize) -> u16 {
    let data_end = 
      usize::from(self.start) + dlen.saturating_sub(1);

    let scr_end = 
      usize::from(self.end).saturating_sub(1);

    u16_or_0(min(data_end, scr_end))
  }

  pub fn get_max_scroll(&self, dlen: usize) -> usize {
    dlen.saturating_sub(self.len())
  }

  pub fn contains(&self, n: u16) -> bool {
    self.start <= n && n <= self.end
  }

  pub fn len16(&self) -> u16 {
    self.end.saturating_sub(self.start)
  }

  pub fn len(&self) -> usize {
    usize::from(self.len16())
  }
}

#[derive(Clone, Debug)]
pub struct PageRange {

  pub scroll: Range16,
  pub text:   Range16,
  pub page:   Range16,
}

impl PageRange {

  pub fn get_data_end(&self, dlen: usize) -> u16 {
    self.text.get_data_end(dlen)
  }

  pub fn get_max_scroll(&self, dlen: usize) -> usize {
    self.text.get_max_scroll(dlen)
  }
}

#[derive(Clone)]
pub struct Page {

  // scroll <= text <= page
  pub scroll:  Rect, 
  pub text:    Rect,
  pub page:    Rect,
} 

impl Dim for Page {

  fn w(&self) -> usize {
    self.page.w
  }

  fn h(&self) -> usize {
    self.page.h
  }
}

impl Page {
  pub fn new(rect: &Rect) -> Self {
    Self {
      page: rect.clone(),
      text: rect.clone(),
      scroll: rect.clone(),
    }
  }

  pub fn text(mut self, x: u16, y: u16) -> Self {
    self.text = self.page.crop_x(x).crop_y(y);
    self.scroll = self.text.clone();
    self
  }

  pub fn scroll(mut self, x: u16, y: u16) -> Self {
    self.scroll = self.text.crop_x(x).crop_y(y);
    self
  }

  pub fn pos(&self) -> Pos {
    Pos::from(&self.text)
  }


  pub fn row(&self, r: u16) -> Self {
    Self {
      scroll: self.scroll.row(r),
      text: self.text.row(r),
      page: self.page.row(r),
    }
  }

  pub fn x(&self) -> PageRange {
    PageRange {
      scroll: self.scroll.x(), 
      text: self.text.x(), 
      page: self.page.x()
    }
  }

  pub fn y(&self) -> PageRange {
    PageRange {
      scroll: self.scroll.y(), 
      text: self.text.y(), 
      page: self.page.y()
    }
  }
}
