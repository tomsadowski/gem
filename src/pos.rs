// src/pos.rs

use crate::{
  util::{u16_or_0},
  page::{Rect, Page, PageRange, Range16},
};


#[derive(Clone, Debug)]
pub struct PosCol {
  pub cursor: u16, 
  pub scroll: usize
}
impl Default for PosCol {

  fn default() -> PosCol {
    PosCol {cursor: 0, scroll: 0}
  }
}
impl From<&Range16> for PosCol {

  fn from(rng: &Range16) -> PosCol {
    PosCol {cursor: rng.start, scroll: 0}
  }
}
impl PosCol {

  // index of cursor within its range
  pub fn data_idx(&self, rng: &Range16) -> usize {

    if self.cursor > rng.start {
      let p = self.cursor.saturating_sub(rng.start);
      self.scroll + usize::from(p)

    } else {
      self.scroll
    }
  }


  // returns the start and end of displayable text
  pub fn data_range(&self, rng: &Range16, len: usize) 
    -> (usize, usize) 
  {
    if len < rng.len() {
      (0, len)

    } else {
      let end = (self.scroll + rng.len())
        .min(len);

      (self.scroll, end)
    }
  }


  pub fn move_into(&mut self, 
                   rng: &PageRange, 
                   len: usize) 
  {
    let (start, end) = 
      if len < rng.text.len() {
        self.scroll = 0;
        let end = rng.text.start + u16_or_0(len);
        (rng.text.start, end)

      } else {
        (rng.text.start, rng.scroll.end)
      };

    if self.cursor < start {
      self.cursor = start;

    } else if self.cursor >= end {
      self.cursor = end;
    }
  }


  pub fn move_backward( &mut self, 
                        rng: &PageRange, 
                        mut step: u16) -> bool
  {
    match (
      self.cursor == rng.text.start, 
      self.scroll == usize::MIN) 
    {
      // nowhere to go, nothing to change
      (true, true) => {
        return false
      }
      // move scroll
      (true, false) => {

        if usize::from(step) < self.scroll  {
          self.scroll -= usize::from(step);

        } else {
          self.scroll = usize::MIN;
        }
      }
      // move cursor
      (false, true) => {

        if rng.text.start + step <= self.cursor {
          self.cursor -= step;

        } else {
          self.cursor = rng.text.start;
        }
      }
      // move cursor and maybe scroll
      (false, false) => {

        if rng.scroll.start + step <= self.cursor {
          self.cursor -= step;

        } else if rng.scroll.start == self.cursor {

          if usize::from(step) <= self.scroll {
            self.scroll -= usize::from(step);

          } else {
            step = step
              .saturating_sub(
                u16_or_0(self.scroll));
            self.scroll = usize::MIN;
            self.move_backward(rng, step);
          }

        } else {
          step = step.saturating_sub(
            self.cursor.saturating_sub(
              rng.scroll.start));
          self.cursor = rng.scroll.start;
          self.move_backward(rng, step);
        }
      }
    }
    return true
  }


  pub fn move_forward(&mut self,
                      rng: &PageRange, 
                      dlen: usize,
                      mut step: u16 ) -> bool
  {
    let screen_data_end = rng.get_data_end(dlen);
    let max_scroll      = rng.get_max_scroll(dlen);

    match (self.cursor == screen_data_end, 
           self.scroll == max_scroll) 
    {
      // nowhere to go, nothing to change
      (true, true) => {
        return false
      }
      // move scroll
      (true, false) => {

        if self.scroll + usize::from(step) >= max_scroll {
          self.scroll += usize::from(step);

        } else {
          self.scroll = max_scroll;
        }
      }
      // move cursor
      (false, true) => {

        if self.cursor + step <= screen_data_end {
          self.cursor += step;

        } else {
          self.cursor = screen_data_end;
        }
      }
      (false, false) => {

        if self.cursor + step <= rng.scroll.end {
          self.cursor += step;

        } else if self.cursor == rng.scroll.end {
          if self.scroll + usize::from(step) <= 
              max_scroll 
          {
            self.scroll += usize::from(step);

          } else {
            step = step.saturating_sub(u16_or_0(
              max_scroll.saturating_sub(self.scroll)));
            self.scroll = max_scroll;
            self.move_forward(rng, dlen, step);
          }
        } else {
          step = step.saturating_sub(
            rng.scroll.end.saturating_sub(self.cursor));
          self.cursor = rng.scroll.end;
          self.move_forward(rng, dlen, step);
        }
      }
    }
    return true
  }
}


#[derive(Clone, Debug)]
pub struct Pos {
  pub x: PosCol,
  pub y: PosCol, 
}
impl Default for Pos {
  fn default() -> Pos {
    Pos {
      x: PosCol::default(),
      y: PosCol::default()
    }
  }
}
impl From<&Rect> for Pos {
  fn from(rect: &Rect) -> Pos {
    Pos {
      x: PosCol::from(&rect.x()),
      y: PosCol::from(&rect.y())
    }
  }
}
impl Pos {

  pub fn move_up( &mut self, 
                  page: &Page, 
                  step: u16) -> bool
  {
    self.y.move_backward(&page.y(), step)
  }


  pub fn move_down(&mut self,
                    page: &Page, 
                    dlen: usize,
                    step: u16 ) -> bool
  {
    self.y.move_forward(&page.y(), dlen, step)
  }


  pub fn move_left( &mut self, 
                    page: &Page, 
                    step: u16) -> bool
  {
    self.x.move_backward(&page.x(), step)
  }


  pub fn move_right(&mut self,
                    page: &Page, 
                    dlen: usize,
                    step: u16 ) -> bool
  {
    self.x.move_forward(&page.x(), dlen, step)
  }
}
