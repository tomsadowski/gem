// src/reader.rs

use crate::{
    screen::{Screen},
    pos::{Pos},
    util::{wrap},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
};
use std::{
    io::{self, Write},
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

pub trait TextDim {
    fn y_len(&self) -> usize;
    fn x_len(&self, y: usize) -> Option<usize>;
}

pub struct DisplayDoc {
    pub scr: Screen,
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
    pub fn new(src: Vec<DisplayText>, scr: &Screen) -> Self {
        let txt = wrap_list(&src, scr.outer.w);
        Self {txt, src, scr: scr.clone()}
    }

    pub fn default(scr: &Screen) -> Self {
        Self {src: vec![], txt: vec![], scr: scr.clone()}
    }

    pub fn resize(&mut self, scr: &Screen) {
        self.scr = scr.clone();
        self.txt = wrap_list(&self.src, self.scr.outer.w);
    }

    pub fn select(&self, pos: &Pos) -> Option<usize> {
        let idx = pos.y().data_idx(&self.scr.outer.y());
        self.txt.get(idx).map(|(u, _)| *u)
    }

    pub fn update_view(&mut self, pos: &Pos) -> io::Result<()> {
        let scroll = pos.y().scroll;
        self.scr.clear();
        for ((idx, txt), line) in 
            (&self.txt[scroll..]).iter().zip(&mut self.scr.buf) 
        {
            line.queue(SetForegroundColor(self.src[*idx].color))?
                .queue(Print(txt))?;
        }
        Ok(())
    }

    pub fn view(&self, writer: &mut impl Write) -> io::Result<()> {
        self.scr.view(writer)
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
