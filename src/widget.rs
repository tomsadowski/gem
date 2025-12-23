// gem/src/widget
// backend agnostic
use crate::util;
use crossterm::{
    QueueableCommand, cursor, style,
    style::Colors,
};
use std::{
    io::{self, Stdout},
    cmp::min,
};

// associates implementor with a color
pub trait GetColors {
    fn getcolors(&self) -> Colors;
}
// a rectangle specified by a point and some lengths
#[derive(Clone, Debug)]
pub struct Rect {
    pub x: u16, pub y: u16, pub w: u16, pub h: u16,
}
impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {x: x, y: y, w: w, h: h}
    }
}
// scroll over data when cursor position is at a limit
// defined by rect
#[derive(Clone, Debug)]
pub struct ScrollingCursor {
    pub scroll: usize,
    pub maxscroll: usize,
    pub cursor: u16,
    pub rect: Rect,
}
impl ScrollingCursor {
    // sets limits given length of text and rect
    pub fn new(textlength: usize, rect: &Rect) -> Self {
        let len = match u16::try_from(textlength) {
            Ok(t) => t, _ => u16::MAX,
        };
        match len < rect.h {
            // no scrolling allowed
            true => Self {
                rect: Rect::new(rect.x, rect.y, rect.w, len),
                cursor: rect.y, 
                scroll: 0, 
                maxscroll: 0,
            },
            // scrolling allowed
            false => Self {
                rect: rect.clone(),
                cursor: rect.y, 
                scroll: 0, 
                maxscroll: textlength - usize::from(rect.h),
            },
        }
    }
    // like Self::new but tries to preserve scroll
    pub fn resize(&mut self, textlength: usize, rect: &Rect) {
        let len = match u16::try_from(textlength) {
            Ok(t) => t, _ => u16::MAX,
        };
        match len < rect.h {
            // no scrolling allowed
            true => {
                self.rect = Rect::new(rect.x, rect.y, rect.w, len);
                self.scroll = 0;
                self.maxscroll = 0;
            },
            // scrolling allowed
            false => {
                self.rect = rect.clone();
                self.scroll = min(self.scroll, self.maxscroll);
                self.maxscroll = textlength - usize::from(rect.h);
            },
        }
        self.cursor = (self.rect.y + self.rect.h - 1) / 2;
    }
    // scroll up when cursor is at highest position
    pub fn moveup(&mut self, step: u16) -> bool {
        let scrollstep = usize::from(step);
        if (self.rect.y + step) <= self.cursor {
            self.cursor -= step;
            true
        } else if usize::MIN + scrollstep <= self.scroll {
            self.scroll -= scrollstep;
            true
        } else {
            false
        }
    }
    // scroll down when cursor is at lowest position
    pub fn movedown(&mut self, step: u16) -> bool {
        let scrollstep = usize::from(step);
        if (self.cursor + step) <= (self.rect.y + self.rect.h - 1) {
            self.cursor += step;
            true 
        } else if (self.scroll + scrollstep) <= self.maxscroll {
            self.scroll += scrollstep;
            true
        } else {
            false
        }
    }
    // returns the start and end of displayable text
    pub fn slicebounds(&self) -> (usize, usize) {
        let start = self.scroll;
        let end = self.scroll + usize::from(self.rect.h);
        (start, end)
    }
    // index of cursor within its rect
    pub fn index(&self) -> usize {
        usize::from(self.cursor - self.rect.y)
    }
}
// enables the selection of metadata (T) behind formatted text.
#[derive(Clone, Debug)]
pub struct Selector<T> {
    rect: Rect,
    wrap: bool,
    pub cursor: ScrollingCursor,
    source: Vec<(T, String)>,
    display: Vec<(usize, String)>,
} 
impl<T: Clone + GetColors> Selector<T> {
    pub fn new(rect: &Rect, source: Vec<(T, String)>, wrap: bool) -> Self {
        let display = match wrap {
            true => util::wraplist(&source, rect.w),
            false => util::cutlist(&source, rect.w),
        };
        return Self {
            rect: rect.clone(),
            wrap: wrap,
            cursor: ScrollingCursor::new(display.len(), &rect),
            source: source,
            display: display,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.display = match self.wrap {
            true => util::wraplist(&self.source, rect.w),
            false => util::cutlist(&self.source, rect.w),
        };
        self.cursor.resize(self.display.len(), rect);
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        let (a, b) = self.cursor.slicebounds();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(self.rect.x, self.rect.y + j as u16))?
                .queue(style::SetColors(self.source[*i].0.getcolors()))?
                .queue(style::Print(text.as_str()))?;
        }
        stdout.queue(cursor::MoveTo(0, self.cursor.cursor))?;
        Ok(())
    }
    pub fn selectundercursor(&self) -> &T {
        &self.source[self.display[self.cursor.index()].0].0
    }
} 
