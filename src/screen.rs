// screen
use std::cmp::min;

pub fn u16_or_0(u: usize) -> u16 {
    u16::try_from(u).unwrap_or(u16::MIN)
}
#[derive(Clone, Debug)]
pub struct ScreenRange {
    pub start:  u16, 
    pub end:    u16
}
impl ScreenRange {
    // if for some reason a > b, just swap them
    pub fn new(start: u16, end: u16) -> ScreenRange {
        if start > end {
            ScreenRange {start: end, end: start}
        } else {
            ScreenRange {start: start, end: end}
        }
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
pub struct Screen {
    pub x: ScreenRange,
    pub y: ScreenRange,
}
impl Screen {
    pub fn origin(w: u16, h: u16) -> Screen {
        Screen {
            x: ScreenRange::new(0, w), 
            y: ScreenRange::new(0, h)}
    }
    pub fn crop_y(&self, step: u16) -> Screen {
        let screen = self.clone();
        screen.crop_north(step).crop_south(step)
    }
    pub fn crop_x(&self, step: u16) -> Screen {
        let screen = self.clone();
        screen.crop_east(step).crop_west(step)
    }
    pub fn crop_south(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if step < screen.y.len16() {
            screen.y.end -= step;
        }
        screen
    }
    pub fn crop_east(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if step < screen.x.len16() {
            screen.x.end -= step;
        }
        screen
    }
    pub fn crop_north(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if step * 2 < screen.y.len16() {
            screen.y.start += step;
        }
        screen
    }
    pub fn crop_west(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if step * 2 < screen.x.len16() {
            screen.x.start += step;
        }
        screen
    }
}
#[derive(Clone, Debug)]
pub struct DataScreenRange {
    pub inner: ScreenRange,
    pub outer: ScreenRange,
}
#[derive(Clone, Debug)]
pub struct DataScreen {
    pub inner: Screen,
    pub outer: Screen,
}
impl DataScreen {
    pub fn new(outer: &Screen, x: u16, y: u16) -> DataScreen {
        Self {
            inner: outer.crop_x(x).crop_y(y),
            outer: outer.clone(),
        }
    }
    pub fn get_x_rng(&self) -> DataScreenRange {
        DataScreenRange {
            inner: self.inner.x.clone(), 
            outer: self.outer.x.clone()
        }
    }
    pub fn get_y_rng(&self) -> DataScreenRange {
        DataScreenRange {
            inner: self.inner.y.clone(), 
            outer: self.outer.y.clone()
        }
    }
}
#[derive(Clone, Debug)]
pub struct PosCol {
    pub cursor: u16, 
    pub scroll: usize
}
impl PosCol {
    pub fn origin(rng: &ScreenRange) -> PosCol {
        PosCol {cursor: rng.start, scroll: 0}
    }
    // index of cursor within its range
    pub fn data_idx(&self, rng: &ScreenRange) -> usize {
        if self.cursor > rng.start {
            let p = self.cursor.saturating_sub(rng.start);
            self.scroll + usize::from(p)
        } else {
            self.scroll
        }
    }
    // returns the start and end of displayable text
    pub fn data_range(&self, rng: &ScreenRange, len: usize) -> (usize, usize) {
        if len < rng.len() {
            (0, len)
        } else {
            (self.scroll, min(self.scroll + rng.len(), len))
        }
    }
    // called when screen resizes
    pub fn fit(rng: &ScreenRange, idx: usize, len: usize) -> PosCol {
        let rng_len = rng.len();
        let max_scroll = len.saturating_sub(rng_len);
        if idx < rng_len {
            let cursor = rng.start + u16_or_0(idx);
            PosCol {
                cursor: cursor, 
                scroll: 0
            }
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
    pub fn move_into(&mut self, dscr: &DataScreenRange, len: usize) {
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
                            dscr: &DataScreenRange, 
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
                            dscr: &DataScreenRange, 
                            dlen: usize,
                            mut step: u16 ) -> bool
    {
        let screen_data_end = u16_or_0(min(
            usize::from(dscr.outer.start) + dlen, 
            usize::from(dscr.outer.end)));
        let max_scroll = dlen.saturating_sub(dscr.outer.len());
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
    pub fn origin(screen: &Screen) -> Pos {
        Pos {
            x: PosCol::origin(&screen.x),
            y: PosCol::origin(&screen.y)}
    }
    pub fn move_left(&mut self, dscr: &DataScreen, step: u16) -> bool {
        self.x.move_backward(&dscr.get_x_rng(), step)
    }
    pub fn move_right(  &mut self,
                        dscr: &DataScreen, 
                        data: &Vec<usize>,
                        step: u16 ) -> bool
    {
        let x_len = {
            let idx = self.y.data_idx(&dscr.outer.y);
            if idx >= data.len() {0} 
            else {data[idx]}
        };
        self.x.move_forward(&dscr.get_x_rng(), x_len, step)
    }
    pub fn move_up( &mut self,
                    dscr: &DataScreen, 
                    data: &Vec<usize>,
                    step: u16 ) -> bool
    {
        if self.y.move_backward(&dscr.get_y_rng(), step) {
            self.move_into_x(dscr, data); true
        } else {false}
    }
    pub fn move_down(   &mut self,
                        dscr: &DataScreen, 
                        data: &Vec<usize>,
                        step: u16 ) -> bool
    {
        if self.y.move_forward(&dscr.get_y_rng(), data.len(), step) {
            self.move_into_x(dscr, data); true
        } else {false}
    }
    pub fn move_into_x(&mut self, dscr: &DataScreen, data: &Vec<usize>) {
        let idx = {
            let idx1 = self.y.data_idx(&dscr.outer.y);
            let idx2 = data.len().saturating_sub(1);
            min(idx1, idx2)
        };
        self.x.move_into(&dscr.get_x_rng(), data[idx]);
    }
    pub fn move_into_y(&mut self, dscr: &DataScreen, data: &Vec<usize>) {
        self.y.move_into(&dscr.get_y_rng(), data.len());
    }
    pub fn get_ranges(&self, dscr: &DataScreen, data: &Vec<usize>) 
        -> Vec<(u16, usize, usize, usize)>
    {
        let mut vec: Vec<(u16, usize, usize, usize)> = vec![];
        let (start, end) = self.y.data_range(&dscr.outer.y, data.len());
        for (e, i) in (start..end).into_iter().enumerate() {
            let (a, b)  = self.x.data_range(&dscr.outer.x, data[i]);
            let scr_idx = dscr.outer.y.start + (e as u16);
            vec.push((scr_idx, i, a, b));
        }
        vec
    }
}
