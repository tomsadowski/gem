// src/screen.rs

use crate::{
    util::{u16_or_0},
};
use crossterm::{
    QueueableCommand,
    terminal::{Clear, ClearType},
    cursor::{MoveTo},
    style::{Print},
};
use std::{
    io::{self, Write},
    cmp::min,
};

#[derive(Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: usize,
    pub h: usize,
}
impl Rect {
    pub fn new(w: u16, h: u16) -> Self {
        let w = usize::from(w);
        let h = usize::from(h);
        Self {x: 0, y: 0, w, h}
    }

    pub fn row(&self, r: u16) -> Rect {
        Rect {
            x: self.x, 
            y: self.y + r,
            h: 1,
            w: self.w
        }
    }

    pub fn x(&self) -> Range16 {
        Range16 {
            start: self.x, end: self.x + u16_or_0(self.w)
        }
    }

    pub fn y(&self) -> Range16 {
        Range16 {
            start: self.y, end: self.y + u16_or_0(self.h)
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w; self.h = h;
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

    pub fn get_page(&self) -> Page {
        Page::new(self)
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
            Range16 {start: end, end: start}
        } else {
            Range16 {start: start, end: end}
        }
    }

    pub fn get_data_end(&self, dlen: usize) -> u16 {
        let data_end = usize::from(self.start) + dlen.saturating_sub(1);
        let scr_end  = usize::from(self.end).saturating_sub(1);
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
pub struct ScreenRange {
    pub inner: Range16,
    pub outer: Range16,
}
impl ScreenRange {
    pub fn get_data_end(&self, dlen: usize) -> u16 {
        self.outer.get_data_end(dlen)
    }

    pub fn get_max_scroll(&self, dlen: usize) -> usize {
        self.outer.get_max_scroll(dlen)
    }
}

#[derive(Clone)]
pub struct Frame {
    pub inner:  Rect, 
    pub outer:  Rect,
} 
impl Frame {
    pub fn new(rect: &Rect, x: u16, y: u16) -> Frame {
        let outer = rect.clone();
        let inner = outer.crop_x(x).crop_y(y);
        Self {
            outer, inner,
        }
    }

    pub fn row(&self, r: u16) -> Frame {
        let inner = self.inner.row(r);
        let outer = self.outer.row(r);
        Frame {inner, outer}
    }


    pub fn x(&self) -> ScreenRange {
        ScreenRange {
            inner: self.inner.x(), 
            outer: self.outer.x()
        }
    }

    pub fn y(&self) -> ScreenRange {
        ScreenRange {
            inner: self.inner.y(), 
            outer: self.outer.y()
        }
    }

    pub fn get_page(&self) -> Page {
        Page::new(&self.outer)
    }
}

#[derive(Clone)]
pub struct Page {
    pub rect: Rect,
    pub buf:  Vec<Vec<u8>>
} 
impl Page {
    pub fn new(rect: &Rect) -> Page {
        Self {
            buf: vec![vec![u8::MIN; rect.w]; rect.h], 
            rect: rect.clone(),
        }
    }
    pub fn view(&self, writer: &mut impl Write) -> io::Result<()> {
        let mut y = self.rect.y;
        writer.queue(MoveTo(self.rect.x, y))?;
        for row in &self.buf {
            if let Ok(c) = std::str::from_utf8(&row) {
                writer
                    .queue(MoveTo(self.rect.x, y))?
                    .queue(Clear(ClearType::CurrentLine))?
                    .queue(Print(c))?;
                y += 1;
            }
        }
        Ok(())
    }
}
