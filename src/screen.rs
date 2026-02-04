// screen
use std::cmp::min;

#[derive(Clone, Debug)]
pub struct ScreenRange {
    pub start: u16, 
    pub end: u16
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
    pub fn len(&self) -> usize {
        usize::from(self.end.saturating_sub(self.start))
    }
}
#[derive(Clone, Debug)]
pub struct Screen {
    pub x: u16, 
    pub y: u16,
    pub w: u16, 
    pub h: u16
}
impl Screen {
    pub fn origin(w: u16, h: u16) -> Screen {
        Screen {x: 0, y: 0, w: w, h: h}
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
        if step < screen.h {
            screen.h -= step;
        }
        screen
    }
    pub fn crop_east(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if step < screen.w {
            screen.w -= step;
        }
        screen
    }
    pub fn crop_north(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if usize::from(step) * 2 < screen.get_y_rng().len() {
            screen.y += step;
            screen.h -= step;
        }
        screen
    }
    pub fn crop_west(&self, step: u16) -> Screen {
        let mut screen = self.clone();
        if usize::from(step) * 2 < screen.get_x_rng().len() {
            screen.x += step;
            screen.w -= step;
        }
        screen
    }
    pub fn get_rngs(&self) -> (ScreenRange, ScreenRange) {
        (self.get_x_rng(), self.get_y_rng())
    }
    pub fn get_x_rng(&self) -> ScreenRange {
        ScreenRange {start: self.x, end: self.x + self.w}
    }
    pub fn get_y_rng(&self) -> ScreenRange {
        ScreenRange {start: self.y, end: self.y + self.h}
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
            inner: self.inner.get_x_rng(), 
            outer: self.outer.get_x_rng()
        }
    }
    pub fn get_y_rng(&self) -> DataScreenRange {
        DataScreenRange {
            inner: self.inner.get_y_rng(), 
            outer: self.outer.get_y_rng()
        }
    }
}
#[derive(Clone, Debug)]
pub struct PosCol {
    pub cursor: u16, 
    pub scroll: usize
}
impl PosCol {
    pub fn fit( rng: &ScreenRange,
                idx: usize,
                len: usize  ) -> PosCol 
    {
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
    pub fn move_into(   &mut self,
                        dscr: &DataScreenRange, 
                        len: usize  )
    {
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
                            dscr:   &DataScreenRange, 
                            step:   u16 ) -> bool
    {
        let mut step = step;
        match (self.cursor == dscr.outer.start, self.scroll == usize::MIN) {
            // nowhere to go, nothing to change
            (true, true) => {
                return false
            }
            // move data point
            (true, false) => {
                if usize::from(step) < self.scroll  {
                    self.scroll -= usize::from(step);
                } else {
                    self.scroll = usize::MIN;
                }
            }
            // move screen point
            (false, true) => {
                if dscr.outer.start + step <= self.cursor {
                    self.cursor -= step;
                } else {
                    self.cursor = dscr.outer.start;
                }
            }
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
                    step -= self.cursor - dscr.inner.start;
                    self.cursor = dscr.inner.start;
                    self.move_backward(dscr, step);
                }
            }
        }
        return true
    }
    pub fn move_forward(    &mut self,
                            dscr:       &DataScreenRange, 
                            dlength:    usize,
                            step:       u16 ) -> bool
    {
        let screen_data_end = u16_or_0(min(
            usize::from(dscr.outer.start) + dlength, 
            usize::from(dscr.outer.end)));
        let mut step = step;
        let screen_len = dscr.outer.len();
        let max_scroll = dlength.saturating_sub(screen_len);
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
                        self.move_forward(dscr, dlength, step);
                    }
                } else {
                    let diff = dscr.inner.end.saturating_sub(self.cursor);
                    step = step.saturating_sub(diff);
                    self.cursor = dscr.inner.end;
                    self.move_forward(dscr, dlength, step);
                }
            }
        }
        return true
    }
}
#[derive(Clone, Debug)]
pub struct Pos {
    pub x_cursor: u16, 
    pub y_cursor: u16,
    pub x_scroll: usize, 
    pub y_scroll: usize
}
impl Pos {
    pub fn origin(screen: &Screen) -> Pos {
        Pos {
            x_cursor: screen.x, 
            y_cursor: screen.y, 
            x_scroll: 0, 
            y_scroll: 0
        }
    }
    pub fn from_cols(x: &PosCol, y: &PosCol) -> Pos {
        Pos {
            x_cursor: x.cursor, 
            y_cursor: y.cursor,
            x_scroll: x.scroll, 
            y_scroll: y.scroll,
        }
    }
    pub fn get_x_col(&self) -> PosCol {
        PosCol {
            cursor: self.x_cursor, 
            scroll: self.x_scroll
        }
    }
    pub fn get_y_col(&self) -> PosCol {
        PosCol {
            cursor: self.y_cursor, 
            scroll: self.y_scroll
        }
    }
    pub fn get_cols(&self) -> (PosCol, PosCol) {
        (self.get_x_col(), self.get_y_col())
    }
    pub fn set_y_col(&mut self, y_col: &PosCol) {
        self.y_cursor = y_col.cursor;
        self.y_scroll = y_col.scroll;
    }
    pub fn set_x_col(&mut self, x_col: &PosCol) {
        self.x_cursor = x_col.cursor;
        self.x_scroll = x_col.scroll;
    }
    pub fn move_left(&mut self, dscr: &DataScreen, step: u16) -> bool {
        let mut x_col = self.get_x_col();
        if x_col.move_backward(&dscr.get_x_rng(), step) {
            self.set_x_col(&x_col); true
        } else {false}
    }
    pub fn move_right(  &mut self,
                        dscr:   &DataScreen, 
                        data:   &Vec<usize>,
                        step:   u16 ) -> bool
    {
        let (mut x_col, y_col) = self.get_cols();
        let y_outer = &dscr.outer.get_y_rng();
        let x_len = {
            let idx = data_idx(&y_outer, &y_col);
            if idx >= data.len() {0} 
            else {data[idx]}
        };
        if x_col.move_forward(&dscr.get_x_rng(), x_len, step) {
            self.set_x_col(&x_col); true
        } else {false}
    }
    pub fn move_up( &mut self,
                    dscr:   &DataScreen, 
                    data:   &Vec<usize>,
                    step:   u16 ) -> bool
    {
        let mut y_col = self.get_y_col();
        if y_col.move_backward(&dscr.get_y_rng(), step) {
            self.set_y_col(&y_col);
            self.move_into_x(dscr, data);
            true
        } else {false}
    }
    pub fn move_down(   &mut self,
                        dscr:   &DataScreen, 
                        data:   &Vec<usize>,
                        step:   u16 ) -> bool
    {
        let mut y_col = self.get_y_col();
        if y_col.move_forward(&dscr.get_y_rng(), data.len(), step) {
            self.set_y_col(&y_col);
            self.move_into_x(dscr, data);
            true
        } else {false}
    }
    pub fn move_into_x(&mut self, dscr: &DataScreen, data: &Vec<usize>) {
        let (mut x_col, y_col) = self.get_cols();
        let idx = {
            let y_outer = dscr.outer.get_y_rng();
            let idx1 = data_idx(&y_outer, &y_col);
            let idx2 = data.len().saturating_sub(1);
            min(idx1, idx2)
        };
        x_col.move_into(&dscr.get_x_rng(), data[idx]);
        self.set_x_col(&x_col);
    }
    pub fn move_into_y(&mut self, dscr: &DataScreen, data: &Vec<usize>) {
        let mut y_col = self.get_y_col();
        let y_len = data.len();
        y_col.move_into(&dscr.get_y_rng(), y_len);
        self.set_y_col(&y_col);
    }
}
// index of cursor within its range
pub fn data_idx(rng: &ScreenRange, col: &PosCol) -> usize {
    if col.cursor > rng.start {
        let diff = usize::from(col.cursor.saturating_sub(rng.start));
        col.scroll + diff
    } else {
        col.scroll
    }
}
pub fn u16_or_0(u: usize) -> u16 {
    u16::try_from(u).unwrap_or(u16::MIN)
}
// returns the start and end of displayable text
pub fn data_range(  rng: &ScreenRange, 
                    pos: &PosCol, 
                    len: usize  ) -> (usize, usize) 
{
    if len < rng.len() {
        (0, len)
    } else {
        (pos.scroll, min(pos.scroll + rng.len(), len))
    }
}
pub fn get_ranges(  dscr:   &DataScreen, 
                    pos:    &Pos, 
                    data:   &Vec<usize> ) -> Vec<(u16, usize, usize, usize)>
{
    let mut vec: Vec<(u16, usize, usize, usize)> = vec![];
    let (x_col, y_col) = pos.get_cols();
    let (x_outer, y_outer) = dscr.outer.get_rngs();
    let (start, end) = data_range(&y_outer, &y_col, data.len());
    for (e, i) in (start..end).into_iter().enumerate() {
        let (a, b) = data_range(&x_outer, &x_col, data[i]);
        let scr_idx = y_outer.start + (e as u16);
        vec.push((scr_idx, i, a, b));
    }
    vec
}
