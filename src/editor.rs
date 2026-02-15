// src/editor.rs

use crate::{
    pos::{PosCol, Pos, TextDim},
    screen::{Frame, Page},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
};

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
