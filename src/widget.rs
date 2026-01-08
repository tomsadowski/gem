// widget

use crate::{
    util::{self, Rect, Range, Cursor},
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
#[derive(Clone, Debug)]
pub struct CursorText {
    cursor: Cursor,
    text:   String,
    range:  Range,
}
impl CursorText {
    pub fn new(rect: &Rect, source: &str) -> Self {
        let range = rect.horizontal().unwrap();
        Self {
            cursor: Cursor::new(source.len(), &range, 0),
            text:   String::from(source),
            range:  range,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        let range = rect.horizontal().unwrap();
        self.cursor.resize(self.text.len(), &range);
        self.range = range;
    }
    pub fn view(&self, y: u16, mut stdout: &Stdout) -> io::Result<()> {
        stdout.queue(cursor::Hide)?;
        let (a, b) = self.cursor.get_display_range();
        let text = &self.text[a..b]; 
        stdout
            .queue(cursor::MoveTo(
                    self.cursor.get_screen_start(), y))?
            .queue(style::Print(text))?
            .queue(cursor::MoveTo(self.cursor.get_cursor(), y))?
            .queue(cursor::Show)?
            .flush()
    }
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
    pub fn move_left(&mut self, step: usize) -> bool {
        self.cursor.backward(step)
    }
    pub fn move_right(&mut self, step: usize) -> bool {
        self.cursor.forward(step)
    }
    pub fn delete(&mut self) -> bool {
        if self.cursor.is_end() {
            false
        } else {
            self.text.remove(self.cursor.get_index());
            self.cursor.resize(self.text.len(), &self.range);
            true
        }
    }
    pub fn backspace(&mut self) -> bool {
        if !self.cursor.backward(1) {
            false
        } else {
            self.text.remove(self.cursor.get_index());
            self.cursor.resize(self.text.len(), &self.range);
            true
        }
    }
    pub fn insert(&mut self, c: char) -> bool {
        if self.cursor.get_index() + 1 == self.text.len() {
            self.text.push(c);
        } else {
            self.text.insert(self.cursor.get_index(), c);
        }
        self.cursor.resize(self.text.len(), &self.range);
        self.cursor.forward(1);
        true
    }
}
#[derive(Clone, Debug)]
pub struct Pager {
    rect:    Rect,
    cursor:  Cursor,
    source:  Vec<ColoredText>,
    display: Vec<(usize, String)>,
} 
impl Pager {
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
            cursor:  
                Cursor::new(
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
        self.cursor.resize(
            self.display.len(), 
            &rect.verticle().unwrap());
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout.queue(cursor::Hide)?;
        let (a, b) = self.cursor.get_display_range();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(
                        self.rect.x, 
                        self.cursor.get_screen_start() + j as u16))?
                .queue(style::SetForegroundColor(
                        self.source[*i].color))?
                .queue(style::Print(text.as_str()))?;
        }
        stdout
            .queue(cursor::MoveTo(
                self.rect.x, 
                self.cursor.get_cursor()))?
            .queue(cursor::Show)?
            .flush()
    }
    pub fn move_up(&mut self, step: usize) -> bool {
        self.cursor.backward(step)
    }
    pub fn move_down(&mut self, step: usize) -> bool {
        self.cursor.forward(step)
    }
    pub fn select_under_cursor(&self) -> (usize, &str) {
        let index = self.display[self.cursor.get_index()].0;
        (index, &self.source[index].text)
    }
} 
