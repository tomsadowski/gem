// src/screen.rs

use crate::{
  util,
  text::{TextPlane, Linear, Planar}
};
use std::ops::Range;

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
  pub fn x_range(&self) -> Range<u16> {
    Range {start: self.x, end: self.x_end()}
  }
  pub fn y_range(&self) -> Range<u16> {
    Range {start: self.y, end: self.y_end()}
  }
  pub fn x_end(&self) -> u16 {
    self.x + self.w
  }
  pub fn y_end(&self) -> u16 {
    self.y + self.h
  }
  pub fn resize(&mut self, w: u16, h: u16) {
    self.w = w; 
    self.h = h;
  }
  pub fn crop_y(&mut self, step: u16) {
    self.crop_north(step);
    self.crop_south(step);
  }
  pub fn crop_x(&mut self, step: u16) {
    self.crop_east(step);
    self.crop_west(step);
  }
  pub fn crop_south(&mut self, step: u16) {
    if step < self.h {
      self.h -= step;
    }
  }
  pub fn crop_east(&mut self, step: u16) {
    if step < self.w {
      self.w -= step
    }
  }
  pub fn crop_north(&mut self, step: u16) {
    if step * 2 < self.h {
      self.y += step;
      self.h -= step;
    }
  }
  pub fn crop_west(&mut self, step: u16) {
    if step * 2 < self.w {
      self.x += step;
      self.w -= step;
    }
  }
  pub fn south_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.y_end();
    let b = rect.y_end();
    util::safe_range(a, b)
  }
  pub fn east_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.x_end();
    let b = rect.x_end();
    util::safe_range(a, b)
  }
  pub fn north_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.y;
    let b = rect.y;
    util::safe_range(a, b)
  }
  pub fn west_range(&self, rect: &Rect) -> Range<u16> {
    let a = self.x;
    let b = rect.x;
    util::safe_range(a, b)
  }
}
#[derive(Clone, Debug, Default)]
pub struct PlaneView {
  x: LineView,
  y: LineView,
}
impl PlaneView {
  pub fn new(rect: &Rect) -> Self {
    Self {
      x: LineView::new(rect.x, rect.w),
      y: LineView::new(rect.y, rect.h),
    }
  }
  pub fn x_cursor(&self) -> u16 {
    self.x.view_head
  }
  pub fn y_cursor(&self) -> u16 {
    self.y.view_head
  }
  pub fn x_scroll(&self) -> usize {
    self.x.line_start
  }
  pub fn y_scroll(&self) -> usize {
    self.y.line_start
  }
  pub fn resize<P: Planar>(&mut self, plane: &P, rect: &Rect) {
    self.x.resize(plane.x_head(), rect.x, rect.w);
    self.y.resize(plane.y_head(), rect.y, rect.h);
  }
  pub fn update<P: Planar>(&mut self, plane: &P, rect: &Rect) {
    self.x.update(plane.x_head());
    self.y.update(plane.y_head());
  }
}
#[derive(Clone, Debug, Default)]
struct LineView {
  pub line_head:  usize,
  pub line_start: usize,
  pub view_head:  u16,
  pub view_start: u16,
  pub view_size:  u16,
}
impl LineView {
  pub fn new(view_start: u16, view_size: u16) -> Self {
    Self {
      line_start: 0, 
      line_head:  0, 
      view_head:  view_start, 
      view_start, 
      view_size
    }
  }
  // preserve cursor position if it still fits in the new bounds
  pub fn resize(&mut self, 
                new_line_head:  usize, 
                new_view_start: u16, 
                new_view_size:  u16) 
  {
    let cursor_position = self.view_head - self.view_start;

    // go to beginning of line
    if new_line_head < usize::from(new_view_size) {
      self.line_start = 0;
      self.view_start = new_view_start;
      self.view_size  = new_view_size;
      self.line_head  = new_line_head;
      self.view_head  = self.view_start + u16::try_from(self.line_head)
          .expect("We do not have Allah's permission");

    // cursor_position must be lowered to fit within new bounds
    } else if cursor_position > new_view_size - 1 {
      self.view_start = new_view_start;
      self.view_size  = new_view_size;
      self.line_head  = new_line_head;
      self.view_head  = self.view_start + self.view_size - 1;
      self.line_start = self.line_head - usize::from(self.view_size - 1);

    // cursor_position can be preserved
    } else {
      self.view_start = new_view_start;
      self.view_size  = new_view_size;
      self.line_head  = new_line_head;
      self.view_head  = self.view_start + cursor_position;
      self.line_start = self.line_head
        .saturating_sub(usize::from(cursor_position));
    }
  }
  pub fn update(&mut self, new_line_head: usize) {
    // forward
    if new_line_head > self.line_head {
      let diff     = new_line_head - self.line_head;
      let proposed = usize::from(self.view_head) + diff;
      let max = 
        usize::from(self.view_start) + 
        usize::from(self.view_size) - 1;
      // line_start forward
      if proposed >= max {
        self.line_start = self.line_start + proposed - max;
      }
    // backward
    } else if new_line_head < self.line_head {
      let diff     = self.line_head - new_line_head;
      let max_diff = usize::from(self.view_head.saturating_sub(self.view_start));
      // line_start backward
      if diff > max_diff {
        self.line_start = self.line_start.saturating_sub(diff - max_diff);
      }
    }
    self.view_head = self.view_start + 
      u16::try_from(new_line_head - self.line_start)
        .expect("We do not have Allah's permission");
    self.line_head = new_line_head;
  }
}
