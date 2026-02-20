// src/text.rs

use crate::{
    screen::{Frame, Dim, Range16},
    pos::{Pos, TextDim},
    util::{self, wrap},
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
        let idx = pos.y.data_idx(&frame.outer.y());
        self.txt.get(idx).map(|(u, _)| *u)
    }

    pub fn view(    &self, 
                    frm: &Frame, 
                    pos: Option<&Pos>,
                    wrt: &mut impl Write) -> io::Result<()> 
    {
        let (x_scroll, y_scroll) = pos
            .map(|p| (p.x.scroll, p.y.scroll))
            .unwrap_or((0, 0));

        let y_start = self.txt.len().saturating_sub(1).min(y_scroll);
        let y_end   = {
            let a = y_start.saturating_add(frm.outer.h);
            let b = y_start.saturating_add(self.txt.len());
            a.min(b)
        };

        for (y, (i, l)) in self.txt[y_start..y_end].iter().enumerate() {

            wrt.queue(SetForegroundColor(self.src[*i].color))?;
            let y_pos     = util::u16_or_0(y) + frm.outer.y;
            let x_scroll  = l.len().saturating_sub(1).min(x_scroll);
            let mut chars = l.chars().skip(x_scroll);
            let Range16 {start: x_start, end: x_end} = frm.outer.x();

            for x_pos in x_start..x_end {
                let c = chars.next().unwrap_or(' ');
                wrt.queue(MoveTo(x_pos, y_pos))?.queue(Print(c))?;
            }
        }
        Ok(())
    }
} 

pub fn wrap_list(lines: &Vec<Text>, w: usize) 
    -> Vec<(usize, String)> 
{
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

pub fn view_line(   txt: &str,
                    col: Color,
                    frm: &Frame, 
                    y: u16,
                    wrt: &mut impl Write) -> io::Result<()> 
{
    wrt.queue(SetForegroundColor(col))?;
    let mut chars = txt.chars();
    let Range16 {start: x_start, end: x_end} = frm.outer.x();

    for x_pos in x_start..x_end {
        let c = chars.next().unwrap_or(' ');
        wrt.queue(MoveTo(x_pos, y))?.queue(Print(c))?;
    }
    Ok(())
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

    pub fn view(    &self, 
                    frm: &Frame, 
                    pos: &Pos,
                    wrt: &mut impl Write) -> io::Result<()> 
    {
        let x_scroll = pos.x.scroll;
        let y_pos    = pos.y.cursor;

        wrt.queue(SetForegroundColor(self.color))?;
        let x_scroll  = self.txt.len().saturating_sub(1).min(x_scroll);
        let mut chars = self.txt.chars().skip(x_scroll);
        let Range16 {start: x_start, end: x_end} = frm.outer.x();

        for x_pos in x_start..x_end {
            let c = chars.next().unwrap_or(' ');
            wrt.queue(MoveTo(x_pos, y_pos))?.queue(Print(c))?;
        }
        Ok(())
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
        } else {
            false
        }
    }

    pub fn backspace(&mut self, frame: &Frame, pos: &mut Pos) 
        -> bool 
    {
        if self.txt.len() == 0 {
            return false
        }
        let idx = pos.x.data_idx(&frame.outer.x());
        pos.x.move_backward(&frame.x(), 1);
        self.txt.remove(idx);
        if pos.x.cursor + 1 != frame.outer.x().end {
            pos.x.move_forward(&frame.x(), self.txt.len(), 1);
        }
        true
    }

    pub fn insert(&mut self, frame: &Frame, pos: &mut Pos, c: char) 
        -> bool 
    {
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
