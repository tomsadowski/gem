// gem/src/widget
// backend agnostic

use crate::util;
use crossterm::{
    QueueableCommand, cursor, style,
    style::{Colors, Color},
};
use std::{
    io::{self, Stdout},
};

// associates implementor with a color
pub trait GetColors {
    fn getcolors(&self) -> Colors;
}
#[derive(Clone, Debug)]
pub struct DefaultColor {
    colors: Colors,
} 
impl DefaultColor {
    pub fn new() -> Self {
        Self {
            colors: Colors::new(
                Color::Rgb {r: 205, g: 205, b: 205},
                Color::Rgb {r: 0, g: 0, b: 0})
        }
    }
} 
impl GetColors for DefaultColor {
    fn getcolors(&self) -> Colors {
        self.colors
    }
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
    cursor: u16,
    screenstart: u16,
    maxcursor: u16,
    scroll: usize,
    maxscroll: usize,
}
impl ScrollingCursor {
    // sets limits given length of text and rect
    pub fn new<T>(text: &Vec<T>, rect: &Rect) -> Self {
        let len = match u16::try_from(text.len()) {
            Ok(t) => t, 
            _ => u16::MAX,
        };
        let maxcursor = std::cmp::min(len, rect.h);
        Self {
            scroll: 0, 
            screenstart: rect.y,
            cursor: rect.y, 
            maxcursor: maxcursor,
            maxscroll: usize::from(len - maxcursor),
        }
    }
    // like Self::new but tries to preserve scroll
    pub fn resize<T>(&mut self, text: &Vec<T>, rect: &Rect) {
        let len = match u16::try_from(text.len()) {
            Ok(t) => t, 
            _ => u16::MAX,
        };
        self.screenstart = rect.y;
        self.maxcursor = std::cmp::min(len, rect.h);
        self.cursor = (self.screenstart + self.maxcursor - 1) / 2;
        self.maxscroll = usize::from(len - self.maxcursor);
        self.scroll = std::cmp::min(self.scroll, self.maxscroll);
    }
    // scroll up when cursor is at highest position
    pub fn moveup(&mut self, step: u16) -> bool {
        let scrollstep = usize::from(step);
        if (self.screenstart + step) <= self.cursor {
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
        if (self.cursor + step) <= (self.screenstart + self.maxcursor - 1) {
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
    pub fn getdisplayrange(&self) -> (usize, usize) {
        (self.scroll, self.scroll + usize::from(self.maxcursor))
    }
    // index of cursor within its rect
    pub fn getcursor(&self) -> u16 {
        self.cursor
    }
    // index of cursor within its rect
    pub fn getindex(&self) -> usize {
        self.scroll + usize::from(self.cursor - self.screenstart)
    }
}
// enables the selection of metadata (T) behind formatted text.
#[derive(Clone, Debug)]
pub struct Selector<T> {
    pub cursor: ScrollingCursor,
    source: Vec<(T, String)>,
    display: Vec<(usize, String)>,
} 
impl Selector<DefaultColor> {
    pub fn default(rect: &Rect, source: &Vec<String>) -> Self {
        let default = source
            .iter()
            .map(|s| (DefaultColor::new(), s.clone()))
            .collect();
        let display = util::wraplist(&default, rect.w);
        return Self {
            cursor: ScrollingCursor::new(&display, &rect),
            source: default.clone(),
            display: display,
        }
    }
}
impl<T: Clone + GetColors> Selector<T> {
    pub fn new(rect: &Rect, source: &Vec<(T, String)>) -> Self {
        let display = util::wraplist(&source, rect.w);
        return Self {
            cursor: ScrollingCursor::new(&display, &rect),
            source: source.clone(),
            display: display,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.display = util::wraplist(&self.source, rect.w);
        self.cursor.resize(&self.display, rect);
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        let (a, b) = self.cursor.getdisplayrange();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(0, self.cursor.screenstart + j as u16))?
                .queue(style::SetColors(self.source[*i].0.getcolors()))?
                .queue(style::Print(text.as_str()))?;
        }
        stdout.queue(cursor::MoveTo(0, self.cursor.getcursor()))?;
        Ok(())
    }
    pub fn selectundercursor(&self) -> &T {
        &self.source[self.display[self.cursor.getindex()].0].0
    }
} 
