// src/screen.rs

use crate::text::{TextPlane, Linear, Planar};

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
pub struct PlaneView {
  pub x: LineView,
  pub y: LineView,
}
impl PlaneView {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: LineView::new(rect.x, rect.w),
      y: LineView::new(rect.y, rect.h),
    }
  }
  pub fn resize<P: Planar>(&mut self, doc: &P, rect: &Rect) {
    self.x.resize(doc.x_head(), rect.x, rect.w);
    self.y.resize(doc.y_head(), rect.y, rect.h);
  }
  pub fn update<P: Planar>(&mut self, doc: &P, rect: &Rect) {
    self.x.update(doc.x_head());
    self.y.update(doc.y_head());
  }
}
#[derive(Clone, Debug, Default)]
pub struct LineView {
  pub head:   usize,
  pub shift:  usize,
  pub point:  u16,
  pub start:  u16,
  pub size:   u16,
}
impl LineView {
  pub fn new(start: u16, size: u16) -> Self {
    Self {shift: 0, head: 0, point: start, start, size}
  }
  // damage control
  pub fn resize(&mut self, new_head: usize, new_start: u16, new_size: u16) {
    let point_len = self.point - self.start;
    self.start = new_start;
    self.size  = new_size;
    self.head  = new_head;
    if new_head < usize::from(self.size) {
      self.shift = 0;
      self.point = self.start + u16::try_from(self.head).unwrap();
    } else if point_len > self.size - 1 {
      self.point = self.start + self.size - 1;
      self.shift = self.head - usize::from(self.size - 1);
    } else {
      self.point = self.start + point_len;
      self.shift = self.head.saturating_sub(usize::from(point_len));
    }
  }
  pub fn update(&mut self, new_head: usize) {
    // move forward
    if new_head > self.head {
      let diff = new_head - self.head;
      let proposed = usize::from(self.point) + diff;
      let max = usize::from(self.start) + usize::from(self.size) - 1;
      // shift forward
      if proposed >= max {
        self.shift = self.shift + proposed - max;
      }
    // move backward
    } else if new_head < self.head {
      let diff = self.head - new_head;
      let max_diff = usize::from(self.point.saturating_sub(self.start));
      // shift backward
      if diff > max_diff {
        self.shift = self.shift.saturating_sub(diff - max_diff);
      }
    }
    self.point = self.start + u16::try_from(new_head - self.shift).unwrap();
    self.head = new_head;
  }
}
