// widget
// 
// + resize
// + view
// - update

use crate::{
    geometry::{Rect, Cursor, ScrollingCursor},
    util, 
};
use crossterm::{
    QueueableCommand, cursor, style,
    style::{Color},
};
use std::{
    io::{self, Stdout},
};

#[derive(Clone, Debug)]
pub struct ColoredText {
    pub color: Color,
    pub text:  String,
}
impl ColoredText {
    pub fn white(text: &str) -> Self {
        Self {
            color: Color::Rgb {r: 205, g: 205, b: 205},
            text: String::from(text),
        }
    }
    pub fn new(text: &str, color: Color) -> Self {
        Self {
            color: color,
            text: String::from(text),
        }
    }
    pub fn getcolor(&self) -> Color {
        self.color
    }
}

#[derive(Clone, Debug)]
pub struct TyperLine {
    pub cursor: Cursor,
    pub source: String,
}
impl TyperLine {
    pub fn new(rect: &Rect, source: &str) -> Self {
        Self {
            cursor: Cursor::new(source.len(), rect),
            source: String::from(source),
        }
    }
}
#[derive(Clone, Debug)]
pub struct WrappedTyperLine {
    pub cursor: ScrollingCursor,
    source:     String,
    display:    Vec<TyperLine>,
}
impl WrappedTyperLine {
    pub fn new(rect: &Rect, source: &str) -> Self {
        let display: Vec<TyperLine> = util::wrap(source, rect.w)
            .iter()
            .map(|s| TyperLine::new(rect, s))
            .collect();
        return Self {
            cursor: ScrollingCursor::new(display.len(), &rect),
            source: String::from(source),
            display: display,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.display = util::wrap(&self.source, rect.w)
            .iter()
            .map(|s| TyperLine::new(rect, s))
            .collect();
        self.cursor.resize(self.display.len(), rect);
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        let (a, b) = self.cursor.getdisplayrange();
        for (j, i) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(0, 
                        self.cursor.getscreenstart() + j as u16))?
                .queue(style::Print(&i.source))?;
        }
        stdout.queue(cursor::MoveTo(0, self.cursor.getcursor()))?;
        Ok(())
    }
} 
// user selects metadata (T) from wrapped, colored text.
// only scrolls vertically
#[derive(Clone, Debug)]
pub struct Selector {
    pub cursor: ScrollingCursor,
    source:     Vec<ColoredText>,
    display:    Vec<(usize, String)>,
} 
impl Selector {
    pub fn white(rect: &Rect, source: &Vec<String>) -> Self {
        let default: Vec<ColoredText> = source
            .iter()
            .map(|s| ColoredText::white(s))
            .collect();
        let display = util::wraplist(&source, rect.w);
        return Self {
            cursor: ScrollingCursor::new(display.len(), &rect),
            source: default.clone(),
            display: display,
        }
    }
    pub fn new(rect: &Rect, source: &Vec<ColoredText>) -> Self {
        let display = util::wraplist(
            &source.iter().map(|ct| ct.text.clone()).collect(),
            rect.w);
        return Self {
            cursor: ScrollingCursor::new(display.len(), &rect),
            source: source.clone(),
            display: display,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.display = util::wraplist(
            &self.source.iter().map(|ct| ct.text.clone()).collect(),
            rect.w);
        self.cursor.resize(self.display.len(), rect);
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        let (a, b) = self.cursor.getdisplayrange();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(0, 
                        self.cursor.getscreenstart() + j as u16))?
                .queue(style::SetForegroundColor(self.source[*i].color))?
                .queue(style::Print(text.as_str()))?;
        }
        stdout.queue(cursor::MoveTo(0, self.cursor.getcursor()))?;
        Ok(())
    }
    pub fn selectundercursor(&self) -> (usize, &str) {
        let index = self.display[self.cursor.getindex()].0;
        (index, &self.source[index].text)
    }
} 
