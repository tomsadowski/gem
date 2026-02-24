// src/text.rs

use crate::{
    screen::{Frame, Dim, Range16},
    pos::{Pos, PosCol},
    util::{self, wrap, u16_or_0},
};
use crossterm::{
    QueueableCommand,
    style::{Color, SetForegroundColor, Print},
    cursor::MoveTo,
};
use std::io::{self, Write};

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
    pub pos: Pos,
    pub src: Vec<Text>,
    pub txt: Vec<(usize, String)>,
} 
impl Doc {
    pub fn new(src: Vec<Text>, frm: &Frame) -> Self {
        let txt = wrap_list(&src, frm.outer.w);
        let pos = frm.pos();
        Self {pos, txt, src}
    }

    pub fn default() -> Self {
        Self {pos: Pos::default(), src: vec![], txt: vec![]}
    }

    pub fn resize(&mut self, frm: &Frame) {
        self.txt = wrap_list(&self.src, frm.outer.w);
    }

    pub fn select(&self, frm: &Frame) -> Option<usize> {
        let idx = self.pos.y.data_idx(&frm.outer.y());
        self.txt.get(idx).map(|(u, _)| *u)
    }

    fn y(&self) -> usize {
        self.txt.len()
    }

    fn x(&self, y: usize) -> Option<usize> {
        self.txt.get(y).map(|(_, txt)| txt.len())
    }

    pub fn move_left(&mut self, frm: &Frame, step: u16) -> bool {
        self.pos.x.move_backward(&frm.x(), step)
    }

    pub fn move_right(&mut self, frm: &Frame, step: u16) -> bool {
        match self.x(self.pos.y.data_idx(&frm.outer.y())) {
            Some(x) => self.pos.x.move_forward(&frm.x(), x, step),
            None    => false
        }
    }

    pub fn move_up(&mut self, frm: &Frame, step: u16) -> bool {
        if self.pos.y.move_backward(&frm.y(), step) {
            self.move_into_x(frm); true
        } else {false}
    }

    pub fn move_down(&mut self, frm: &Frame, step: u16) -> bool {
        if self.pos.y.move_forward(&frm.y(), self.y(), step) {
            self.move_into_x(frm); true
        } else {false}
    }

    pub fn move_into_x(&mut self, frm: &Frame) {
        let idx = self.pos.y
            .data_idx(&frm.outer.y())
            .min(self.y().saturating_sub(1));
        self
            .x(idx)
            .inspect(|d| self.pos.x.move_into(&frm.x(), *d));
    }

    pub fn view<W: Write>(&self, frm: &Frame, wrt: &mut W) -> io::Result<()> {
        let y_start = self.txt.len().saturating_sub(1).min(self.pos.y.scroll);
        let y_end = y_start
            .saturating_add(frm.outer.h)
            .min(y_start.saturating_add(self.txt.len()));
        for (y, (i, line)) in self.txt[y_start..y_end].iter().enumerate() {
            wrt.queue(SetForegroundColor(self.src[*i].color))?;
            let mut chars = line
                .chars()
                .skip(line.len().saturating_sub(1).min(self.pos.x.scroll));
            let Range16 {start: x_start, end: x_end} = frm.outer.x();
            for x_pos in x_start..x_end {
                wrt
                    .queue(MoveTo(x_pos, u16_or_0(y) + frm.outer.y))?
                    .queue(Print(chars.next().unwrap_or(' ')))?;
            }
        }
        wrt.queue(MoveTo(self.pos.x.cursor, self.pos.y.cursor))?;
        Ok(())
    }
} 

pub fn wrap_list(lines: &Vec<Text>, w: usize) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, l) in lines.iter().enumerate() {
        let v = 
            if l.wrap {
                wrap(&l.text, w)
            } else {
                vec![l.text.clone()]
            };
        for s in v.iter() {
            display.push((i, s.to_string()));
        }
    }
    display
}

pub fn white_line<W: Write>(txt: &str, frm: &Frame, wrt: &mut W) 
    -> io::Result<()> 
{
    wrt.queue(SetForegroundColor(Color::White))?;
    let mut chars = txt.chars();
    let Range16 {start: x_start, end: x_end} = frm.outer.x();
    for x_pos in x_start..x_end {
        wrt
            .queue(MoveTo(x_pos, frm.outer.y))?
            .queue(Print(chars.next().unwrap_or(' ')))?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct Editor {
    pub pos:    PosCol,
    pub txt:    String,
    pub color:  Color,
}
impl Editor {
    pub fn new(frm: &Frame, txt: &str, color: Color) -> Self {
        Self {
            pos:    frm.pos().x,
            color:  color,
            txt:    txt.into(),
        }
    }

    pub fn move_left(&mut self, frm: &Frame, step: u16) -> bool {
        self.pos.move_backward(&frm.x(), step)
    }

    pub fn move_right(&mut self, frm: &Frame, step: u16) -> bool {
        self.pos.move_forward(&frm.x(), self.txt.len(), step)
    }

    pub fn view<W: Write>(&self, frm: &Frame, pos: &Pos, wrt: &mut W) 
        -> io::Result<()> 
    {
        wrt.queue(SetForegroundColor(self.color))?;
        let mut chars = self.txt
            .chars()
            .skip(self.txt.len().saturating_sub(1).min(pos.x.scroll));
        let Range16 {start: x_start, end: x_end} = frm.outer.x();
        for x_pos in x_start..x_end {
            let c = chars.next().unwrap_or(' ');
            wrt.queue(MoveTo(x_pos, pos.y.cursor))?.queue(Print(c))?;
        }
        Ok(())
    }

    pub fn delete(&mut self, frm: &Frame, pos: &mut Pos) -> bool {
        let outer = frm.outer.x();
        let idx = pos.x.data_idx(&outer);
        if idx >= self.txt.len() || self.txt.len() == 0 {
            return false
        }
        self.txt.remove(idx);
        if pos.x.cursor + 1 != outer.end {
            pos.x.move_forward(&frm.x(), self.txt.len(), 1)
        } else {
            false
        }
    }

    pub fn backspace(&mut self, frm: &Frame, pos: &mut Pos) -> bool {
        if self.txt.len() == 0 {
            return false
        }
        let idx = pos.x.data_idx(&frm.outer.x());
        pos.x.move_backward(&frm.x(), 1);
        self.txt.remove(idx);
        if pos.x.cursor + 1 != frm.outer.x().end {
            pos.x.move_forward(&frm.x(), self.txt.len(), 1);
        }
        true
    }

    pub fn insert(&mut self, frm: &Frame, pos: &mut Pos, c: char) -> bool {
        let idx = pos.x.data_idx(&frm.outer.x()) + 1;
        if idx >= self.txt.len() || self.txt.len() == 0 {
            self.txt.push(c);
        } else {
            self.txt.insert(idx, c);
        }
        pos.x.move_forward(&frm.x(), self.txt.len(), 1);
        true
    }
}
