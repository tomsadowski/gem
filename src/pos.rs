// src/pos.rs

use crate::{
    util::{u16_or_0},
    screen::{Rect, Screen, ScreenRange, Range16},
    reader::{TextDim},
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
    pub fn data_idx_cap(&self, rng: &Range16, max: usize) -> usize {
        let idx = if self.cursor > rng.start {
            let p = self.cursor.saturating_sub(rng.start);
            self.scroll + usize::from(p)
        } else {self.scroll};
        min(idx, max)
    }

    // index of cursor within its range
    pub fn data_idx(&self, rng: &Range16) -> usize {
        if self.cursor > rng.start {
            let p = self.cursor.saturating_sub(rng.start);
            self.scroll + usize::from(p)
        } else {self.scroll}
    }

    // returns the start and end of displayable text
    pub fn data_range(&self, rng: &Range16, len: usize) -> (usize, usize) {
        if len < rng.len() {(0, len)} 
        else {(self.scroll, min(self.scroll + rng.len(), len))}
    }

    // called when screen resizes
    pub fn fit(rng: &Range16, idx: usize, len: usize) -> PosCol {
        let rng_len = rng.len();
        let max_scroll = len.saturating_sub(rng_len);
        if idx < rng_len {
            let cursor = rng.start + u16_or_0(idx);
            PosCol {cursor, scroll: 0}
        } else if idx > max_scroll {
            let cursor = rng.start + 
                u16_or_0(idx.saturating_sub(max_scroll));
            PosCol {
                cursor: cursor, 
                scroll: max_scroll
            }
        } else {
            let scroll = idx.saturating_sub(rng_len / 2);
            let cursor = rng.start + 
                u16_or_0(idx.saturating_sub(scroll));
            PosCol {
                cursor: cursor, 
                scroll: scroll
            }
        }
    }

    pub fn move_into(&mut self, dscr: &ScreenRange, len: usize) {
        let (start, end) = 
            if len < dscr.outer.len() {
                self.scroll = 0;
                let len = u16_or_0(len);
                (dscr.outer.start, dscr.outer.start + len)
            } else {
                (dscr.outer.start, dscr.inner.end)
            };
        if self.cursor < start {
            self.cursor = start;
        }
        else if self.cursor >= end {
            self.cursor = end;
        }
    }

    pub fn move_backward(   &mut self, 
                            dscr: &ScreenRange, 
                            mut step: u16) -> bool
    {
        match (self.cursor == dscr.outer.start, self.scroll == usize::MIN) {
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
                if dscr.outer.start + step <= self.cursor {
                    self.cursor -= step;
                } else {
                    self.cursor = dscr.outer.start;
                }
            }
            // move cursor and maybe scroll
            (false, false) => {
                if dscr.inner.start + step <= self.cursor {
                    self.cursor -= step;
                } else if dscr.inner.start == self.cursor {
                    if usize::from(step) <= self.scroll {
                        self.scroll -= usize::from(step);
                    } else {
                        step -= u16_or_0(self.scroll);
                        self.scroll = usize::MIN;
                        self.move_backward(dscr, step);
                    }
                } else {
                    step -= self.cursor.saturating_sub(dscr.inner.start);
                    self.cursor = dscr.inner.start;
                    self.move_backward(dscr, step);
                }
            }
        }
        return true
    }

    pub fn move_forward(    &mut self,
                            dscr: &ScreenRange, 
                            dlen: usize,
                            mut step: u16 ) -> bool
    {
        let screen_data_end = dscr.get_data_end(dlen);
        let max_scroll      = dscr.get_max_scroll(dlen);
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
                if self.cursor + step <= dscr.inner.end {
                    self.cursor += step;
                } else if self.cursor == dscr.inner.end {
                    if self.scroll + usize::from(step) <= max_scroll {
                        self.scroll += usize::from(step);
                    } else {
                        let diff = 
                            u16_or_0(max_scroll.saturating_sub(self.scroll));
                        step = step.saturating_sub(diff);
                        self.scroll = max_scroll;
                        self.move_forward(dscr, dlen, step);
                    }
                } else {
                    let diff = dscr.inner.end.saturating_sub(self.cursor);
                    step = step.saturating_sub(diff);
                    self.cursor = dscr.inner.end;
                    self.move_forward(dscr, dlen, step);
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
    pub fn origin(rect: &Rect) -> Pos {
        Pos {
            x: PosCol::origin(&rect.x()),
            y: PosCol::origin(&rect.y())}
    }

    pub fn x(&self) -> PosCol {
        self.x.clone()
    }

    pub fn y(&self) -> PosCol {
        self.y.clone()
    }

    pub fn move_left(&mut self, scr: &Screen, step: u16) -> bool {
        self.x.move_backward(&scr.x(), step)
    }

    pub fn move_right<T>(&mut self, scr: &Screen, data: &T, step: u16) -> bool
    where T: TextDim
    {
        let idx = self.y.data_idx(&scr.outer.y());
        match data.x_len(idx) {
            Some(x_len) => self.x.move_forward(&scr.x(), x_len, step),
            None => false
        }
    }

    pub fn move_up<T>(&mut self, scr: &Screen, data: &T, step: u16) -> bool
    where T: TextDim
    {
        if self.y.move_backward(&scr.y(), step) {
            self.move_into_x(scr, data); true
        } else {false}
    }

    pub fn move_down<T>(&mut self, scr: &Screen, data: &T, step: u16) -> bool
    where T: TextDim
    {
        if self.y.move_forward(&scr.y(), data.y_len(), step) {
            self.move_into_x(scr, data); true
        } else {false}
    }

    pub fn move_into_x<T>(&mut self, scr: &Screen, data: &T) 
    where T: TextDim
    {
        let idx = {
            let idx1 = self.y.data_idx(&scr.outer.y());
            let idx2 = data.y_len().saturating_sub(1);
            min(idx1, idx2)
        };
        data.x_len(idx).inspect(|d| self.x.move_into(&scr.x(), *d));
    }

    pub fn move_into_y(&mut self, scr: &Screen, data: &Vec<usize>) {
        self.y.move_into(&scr.y(), data.len());
    }
}
