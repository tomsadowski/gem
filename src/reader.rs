// src/reader.rs

use crate::{
    screen::{Frame, Page},
    pos::{Pos, TextDim},
    util::{wrap},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
};

pub struct DisplayText {
    pub color: Color,
    pub text:  String,
    pub wrap:  bool,
}
impl DisplayText {
    pub fn new(text: &str, color: Color, wrap: bool) -> Self {
        Self {text: text.into(), color, wrap}
    }
    pub fn default() -> Self {
        Self {text: "".into(), color: Color::White, wrap: false}
    }
}

pub struct DisplayDoc {
    pub frame: Frame,
    pub src: Vec<DisplayText>,
    pub txt: Vec<(usize, String)>,
} 
impl TextDim for DisplayDoc {
    fn y_len(&self) -> usize {
        self.txt.len()
    }

    fn x_len(&self, y: usize) -> Option<usize> {
        self.txt.get(y).map(|(_, txt)| txt.len())
    }
}
impl DisplayDoc {
    pub fn new(src: Vec<DisplayText>, frame: &Frame) -> Self {
        let txt = wrap_list(&src, frame.outer.w);
        Self {txt, src, frame: frame.clone()}
    }

    pub fn default(frame: &Frame) -> Self {
        Self {src: vec![], txt: vec![], frame: frame.clone()}
    }

    pub fn resize(&mut self, frame: &Frame) {
        self.frame = frame.clone();
        self.txt = wrap_list(&self.src, self.frame.outer.w);
    }

    pub fn select(&self, pos: &Pos) -> Option<usize> {
        let idx = pos.y().data_idx(&self.frame.outer.y());
        self.txt.get(idx).map(|(u, _)| *u)
    }

    pub fn get_page(&self, pos: Option<&Pos>) -> Page {
        let scroll = 
            if let Some(p) = pos {
                p.y().scroll
            } else {0};
        let mut page = self.frame.get_page();
        for ((idx, txt), line) in 
            (&self.txt[scroll..]).iter().zip(&mut page.buf) 
        {
            line.queue(SetForegroundColor(self.src[*idx].color)).unwrap()
                .queue(Print(txt)).unwrap();
        }
        page
    }
} 
pub fn wrap_list(lines: &Vec<DisplayText>, w: usize) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, l) in lines.iter().enumerate() {
        let v = if l.wrap {wrap(&l.text, w)} else {vec![l.text.clone()]};
        for s in v.iter() {display.push((i, s.to_string()));}
    }
    display
}
