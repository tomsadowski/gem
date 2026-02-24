// src/pos.rs

use crate::{
    util::{u16_or_0},
    screen::{Rect, Frame, ScreenRange, Range16},
};
use std::{
    cmp::min,
};

#[derive(Clone, Debug)]
pub struct PosCol {
    pub cursor: u16, 
    pub scroll: usize
}
impl PosCol {
    pub fn origin(rng: &Range16) -> PosCol {
        PosCol {cursor: rng.start, scroll: 0}
    }

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
    pub fn data_range(&self, rng: &Range16, len: usize) -> (usize, usize) {
        if len < rng.len() {(0, len)} 
        else {(self.scroll, min(self.scroll + rng.len(), len))}
    }

    pub fn move_into(&mut self, rng: &ScreenRange, len: usize) {
        let (start, end) = 
            if len < rng.outer.len() {
                self.scroll = 0;
                (rng.outer.start, rng.outer.start + u16_or_0(len))
            } else {
                (rng.outer.start, rng.inner.end)
            };
        if self.cursor < start {
            self.cursor = start;
        }
        else if self.cursor >= end {
            self.cursor = end;
        }
    }

    pub fn move_backward(   &mut self, 
                            rng: &ScreenRange, 
                            mut step: u16) -> bool
    {
        match (self.cursor == rng.outer.start, self.scroll == usize::MIN) {
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
                if rng.outer.start + step <= self.cursor {
                    self.cursor -= step;
                } else {
                    self.cursor = rng.outer.start;
                }
            }
            // move cursor and maybe scroll
            (false, false) => {
                if rng.inner.start + step <= self.cursor {
                    self.cursor -= step;
                } else if rng.inner.start == self.cursor {
                    if usize::from(step) <= self.scroll {
                        self.scroll -= usize::from(step);
                    } else {
                        step = step.saturating_sub(u16_or_0(self.scroll));
                        self.scroll = usize::MIN;
                        self.move_backward(rng, step);
                    }
                } else {
                    step = step.saturating_sub(
                        self.cursor.saturating_sub(rng.inner.start));
                    self.cursor = rng.inner.start;
                    self.move_backward(rng, step);
                }
            }
        }
        return true
    }

    pub fn move_forward(    &mut self,
                            rng: &ScreenRange, 
                            dlen: usize,
                            mut step: u16 ) -> bool
    {
        let screen_data_end = rng.get_data_end(dlen);
        let max_scroll      = rng.get_max_scroll(dlen);
        match (self.cursor == screen_data_end, self.scroll == max_scroll) {
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
                if self.cursor + step <= rng.inner.end {
                    self.cursor += step;
                } else if self.cursor == rng.inner.end {
                    if self.scroll + usize::from(step) <= max_scroll {
                        self.scroll += usize::from(step);
                    } else {
                        step = step.saturating_sub(u16_or_0(
                            max_scroll.saturating_sub(self.scroll)));
                        self.scroll = max_scroll;
                        self.move_forward(rng, dlen, step);
                    }
                } else {
                    step = step.saturating_sub(
                        rng.inner.end.saturating_sub(self.cursor));
                    self.cursor = rng.inner.end;
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
impl Pos {
    pub fn default() -> Pos {
        Pos {
            x: PosCol {cursor: 0, scroll: 0},
            y: PosCol {cursor: 0, scroll: 0}
        }
    }
    pub fn origin(rect: &Rect) -> Pos {
        Pos {
            x: PosCol::origin(&rect.x()),
            y: PosCol::origin(&rect.y())
        }
    }
}
