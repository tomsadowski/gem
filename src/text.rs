// src/text.rs

use crate::{
    screen::{Frame, Page},
    pos::{Pos, TextDim},
    util::{wrap},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
};

pub struct Text {
    pub color: Color,
    pub text:  String,
    pub wrap:  bool,
}
impl Text {
    pub fn new(text: &str, color: Color, wrap: bool) -> Self {
        Self {text: text.into(), color, wrap}
    }
    pub fn default() -> Self {
        Self {text: "".into(), color: Color::White, wrap: false}
    }
}

pub struct Doc {
    pub src: Vec<Text>,
    pub txt: Vec<(usize, String)>,
} 
impl TextDim for Doc {
    fn y_len(&self) -> usize {
        self.txt.len()
    }

    fn x_len(&self, y: usize) -> Option<usize> {
        self.txt.get(y).map(|(_, txt)| txt.len())
    }
}
impl Doc {
    pub fn new(src: Vec<Text>, frame: &Frame) -> Self {
        let txt = wrap_list(&src, frame.outer.w);
        Self {txt, src}
    }

    pub fn default() -> Self {
        Self {src: vec![], txt: vec![]}
    }

    pub fn resize(&mut self, frame: &Frame) {
        self.txt = wrap_list(&self.src, frame.outer.w);
    }

    pub fn select(&self, frame: &Frame, pos: &Pos) -> Option<usize> {
        let idx = pos.y().data_idx(&frame.outer.y());
        self.txt.get(idx).map(|(u, _)| *u)
    }

    pub fn get_page(&self, frame: &Frame, pos: Option<&Pos>) -> Page {
        let scroll = 
            if let Some(p) = pos {
                // TODO, figure out how not to require this check
                std::cmp::min(p.y().scroll, self.txt.len() - 1)
            } else {0};
        let mut page = frame.get_page();
        for ((idx, txt), line) in 
            (&self.txt[scroll..]).iter().zip(&mut page.buf) 
        {
            line.queue(SetForegroundColor(self.src[*idx].color)).unwrap()
                .queue(Print(txt)).unwrap();
        }
        page
    }
} 
pub fn wrap_list(lines: &Vec<Text>, w: usize) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, l) in lines.iter().enumerate() {
        let v = if l.wrap {wrap(&l.text, w)} else {vec![l.text.clone()]};
        for s in v.iter() {display.push((i, s.to_string()));}
    }
    display
}

#[derive(Clone)]
pub struct Editor {
    pub txt:    String,
    pub color:  Color,
}
impl TextDim for Editor {
    fn y_len(&self) -> usize {1}
    fn x_len(&self, _: usize) -> Option<usize> {
        Some(self.txt.len())
    }
}
impl Editor {
    pub fn new(txt: &str, color: Color) -> Self {
        Self {
            color:  color,
            txt:    txt.into(),
        }
    }

    pub fn get_page(&self, frame: &Frame, pos: &Pos) -> Page {
        let scroll = pos.x.scroll;
        let mut page = frame.get_page();
        (&mut page.buf[0])
            .queue(SetForegroundColor(self.color)).unwrap()
            .queue(Print(&self.txt[scroll..])).unwrap();
        page
    }

    pub fn delete(&mut self, frame: &Frame, pos: &mut Pos) -> bool {
        let outer = frame.outer.x();
        let idx = pos.x.data_idx(&outer);
        if idx >= self.txt.len() || self.txt.len() == 0 {
            return false
        }
        self.txt.remove(idx);
        if pos.x.cursor + 1 != outer.end {
            pos.x.move_forward(&frame.x(), self.txt.len(), 1)
        } else {false}
    }

    pub fn backspace(&mut self, frame: &Frame, pos: &mut Pos) -> bool {
        if self.txt.len() == 0 {return false}
        let idx = pos.x.data_idx(&frame.outer.x());
        pos.x.move_backward(&frame.x(), 1);
        self.txt.remove(idx);
        if pos.x.cursor + 1 != frame.outer.x().end {
            pos.x.move_forward(&frame.x(), self.txt.len(), 1);
        }
        true
    }

    pub fn insert(&mut self, frame: &Frame, pos: &mut Pos, c: char) -> bool {
        let idx = pos.x.data_idx(&frame.outer.x()) + 1;
        if idx >= self.txt.len() || self.txt.len() == 0 {
            self.txt.push(c);
        } else {
            self.txt.insert(idx, c);
        }
        pos.x.move_forward(&frame.x(), self.txt.len(), 1);
        true
    }
}
