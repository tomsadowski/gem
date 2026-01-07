// widget

use crate::{
    util::{self, Rect, ScrollingCursor},
};
use crossterm::{
    QueueableCommand, cursor,
    terminal::{self, ClearType},
    style::{self, Color},
};
use std::{
    io::{self, Stdout, Write},
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
// user selects metadata (T) from wrapped, colored text.
// only scrolls vertically
#[derive(Clone, Debug)]
pub struct Selector {
    rect:             Rect,
    scrolling_cursor: ScrollingCursor,
    source:           Vec<ColoredText>,
    display:          Vec<(usize, String)>,
} 
impl Selector {
    pub fn white(rect: &Rect, source: &Vec<String>) -> Self {
        let white: Vec<ColoredText> = source
            .iter()
            .map(|s| ColoredText::white(s))
            .collect();
        Self::new(rect, &white, 0)
    }
    pub fn new(rect: &Rect, source: &Vec<ColoredText>, buf: u8) -> Self {
        let display = util::wrap_list(
            &source.iter().map(|ct| ct.text.clone()).collect(),
            rect.w);
        return Self {
            rect:    rect.clone(),
            scrolling_cursor:  
                ScrollingCursor::new(
                    display.len(), 
                    &rect.verticle().unwrap(), 
                    buf),
            source:  source.clone(),
            display: display,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.rect    = rect.clone();
        self.display = util::wrap_list(
            &self.source.iter().map(|ct| ct.text.clone()).collect(),
            rect.w);
        self.scrolling_cursor.resize(
            self.display.len(), 
            &rect.verticle().unwrap());
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout.queue(cursor::Hide)?;
        let (a, b) = self.scrolling_cursor.get_display_range();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(
                        self.rect.x, 
                        self.scrolling_cursor.get_screen_start() + j as u16))?
                .queue(style::SetForegroundColor(
                        self.source[*i].color))?
                .queue(style::Print(text.as_str()))?;
        }
        stdout
            .queue(cursor::MoveTo(
                self.rect.x, 
                self.scrolling_cursor.get_cursor()))?
            .queue(cursor::Show)?
            .flush()
    }
    pub fn move_up(&mut self, step: usize) -> bool {
        self.scrolling_cursor.move_up(step)
    }
    pub fn move_down(&mut self, step: usize) -> bool {
        self.scrolling_cursor.move_down(step)
    }
    pub fn select_under_cursor(&self) -> (usize, &str) {
        let index = self.display[self.scrolling_cursor.get_index()].0;
        (index, &self.source[index].text)
    }
} 
