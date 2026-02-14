// src/editor.rs

use crate::{
    pos::{PosCol},
    screen::{Screen},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
};
use std::{
    io::{self, Write},
};

#[derive(Clone)]
pub struct Editor {
    scr:    Screen,
    pcol:   PosCol,
    txt:    String,
    color:  Color,
}
impl Editor {
    pub fn new(scr: &Screen, txt: &str, color: Color) -> Self {
        Self {
            color:  color,
            pcol:   PosCol::origin(&scr.outer.x()),
            txt:    txt.into(),
            scr:    scr.clone(),
        }
    }

    pub fn resize(&mut self, scr: &Screen) {
        self.scr = scr.clone();
    }

    pub fn update_view(&mut self) -> io::Result<()> {
        let scroll = self.pcol.scroll;
        self.scr.clear();
        (&mut self.scr.buf[0])
            .queue(SetForegroundColor(self.color))?
            .queue(Print(&self.txt[scroll..]))?;
        Ok(())
    }

    pub fn view(&self, writer: &mut impl Write) -> io::Result<()> {
        self.scr.view(writer)
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
