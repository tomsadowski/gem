// src/editor.rs

use crate::{
    pos::{PosCol},
    screen::{Frame, Page},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
};

#[derive(Clone)]
pub struct Editor {
    scr:    Frame,
    pcol:   PosCol,
    txt:    String,
    color:  Color,
}
impl Editor {
    pub fn new(scr: &Frame, txt: &str, color: Color) -> Self {
        Self {
            color:  color,
            pcol:   PosCol::origin(&scr.outer.x()),
            txt:    txt.into(),
            scr:    scr.clone(),
        }
    }

    pub fn resize(&mut self, scr: &Frame) {
        self.scr = scr.clone();
    }

    pub fn get_page(&self) -> Page {
        let scroll = self.pcol.scroll;
        let mut page = self.scr.get_page();
        (&mut page.buf[0])
            .queue(SetForegroundColor(self.color)).unwrap()
            .queue(Print(&self.txt[scroll..])).unwrap();
        page
    }

    pub fn get_text(&self) -> String {
        self.txt.clone()
    }

    pub fn move_left(&mut self, step: u16) -> bool {
        self.pcol.move_backward(&self.scr.x(), step)
    }

    pub fn move_right(&mut self, step: u16) -> bool {
        self.pcol.move_forward(&self.scr.x(), self.txt.len(), step)
    }

    pub fn delete(&mut self) -> bool {
        let idx = self.pcol.data_idx(&self.scr.outer.x());
        if idx >= self.txt.len() || self.txt.len() == 0 {
            return false
        }
        self.txt.remove(idx);
        if self.pcol.cursor + 1 != self.scr.outer.x().end {
            self.pcol.move_forward(&self.scr.x(), self.txt.len(), 1);
        }
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.txt.len() == 0 {return false}
        let idx = self.pcol.data_idx(&self.scr.outer.x());
        self.pcol.move_backward(&self.scr.x(), 1);
        self.txt.remove(idx);
        if self.pcol.cursor + 1 != self.scr.outer.x().end {
            self.pcol.move_forward(&self.scr.x(), self.txt.len(), 1);
        }
        true
    }

    pub fn insert(&mut self, c: char) -> bool {
        let idx = self.pcol.data_idx(&self.scr.outer.x()) + 1;
        if idx >= self.txt.len() || self.txt.len() == 0 {
            self.txt.push(c);
        } else {
            self.txt.insert(idx, c);
        }
        self.pcol.move_forward(&self.scr.x(), self.txt.len(), 1);
        true
    }
}
