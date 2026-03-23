// src/screen.rs

use crate::text::{CursorDoc, Tape};

#[derive(Clone, Default)]
pub struct Rect {
  pub x: u16,
  pub y: u16,
  pub w: u16,
  pub h: u16,
}
impl Rect {
  pub fn new(w: u16, h: u16) -> Self {
    Self {x: 0, y: 0, w, h}
  }
  pub fn resize(&mut self, w: u16, h: u16) {
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
    if step < rect.h {
      rect.h -= step;
    }
    rect
  }
  pub fn crop_east(&self, step: u16) -> Self {
    let mut rect = self.clone();
    if step < rect.w {
      rect.w -= step
    }
    rect
  }
  pub fn crop_north(&self, step: u16) -> Self {
    let mut rect = self.clone();
    if step * 2 < rect.h {
      rect.y += step;
      rect.h -= step;
    }
    rect
  }
  pub fn crop_west(&self, step: u16) -> Self {
    let mut rect = self.clone();
    if step * 2 < rect.w {
      rect.x += step;
      rect.w -= step;
    }
    rect
  }
}

#[derive(Clone, Debug, Default)]
pub struct Pos {
  pub x: PosCol,
  pub y: PosCol,
}
impl Pos {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: PosCol::new(rect.x, rect.w),
      y: PosCol::new(rect.y, rect.h),
    }
  }
  pub fn update(&mut self, doc: &CursorDoc, rect: &Rect) {
    self.x.update(doc.x(), doc.x_len(), rect.x, rect.w);
    self.y.update(doc.y(), doc.y_len(), rect.y, rect.h);
  }
}
#[derive(Clone, Debug, Default)]
pub struct PosCol {
  pub shift: usize,
  pub point: u16,
}
impl PosCol {
  pub fn new(start: u16, width: u16) -> Self {
    Self {shift: 0, point: start}
  }
  pub fn head(&self) -> usize {
    self.shift + usize::from(self.point)
  }
  pub fn update(&mut self, 
    head: usize, len: usize, 
    start: u16, width: u16) 
  {
    let trimmed_point = usize::from(self.point.saturating_sub(start));
    let trimmed_head = head.saturating_sub(self.shift);
    let window = usize::from(width);

    if window >= len {
      // tape.head < head <= window == width: u16
      self.shift = 0;
      self.point = start + u16::try_from(head)
        .expect("tape.head < tape.len <= window == width: u16");

    } else {
      // move backward
      if trimmed_point >= trimmed_head {
        if self.shift > head {
          self.shift = head;
        }
      // move forward
      } else {
        if trimmed_head >= window {
          self.shift = self.shift + trimmed_head.saturating_sub(trimmed_point);
        }
      }
      // tape.head - shift <= window == width: u16
      self.point = start + 
        u16::try_from(head - self.shift)
        .expect("tape.head - shift <= window == width: u16");
    }
  }
}
