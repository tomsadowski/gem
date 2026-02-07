// text

use crate::{
    screen::{
        self, Screen, ScreenRange, Pos, PosCol, DataScreen, DataScreenRange},
};
use crossterm::{
    QueueableCommand, 
    cursor::{self, MoveTo},
    terminal::{self, ClearType},
    style::{self, Color, SetForegroundColor, Print},
};
use std::{
    io::{self, Stdout, Write},
};

pub fn split_whitespace_once(source: &str) -> (&str, &str) {
    let line = source.trim();
    let (a, b) = {
        if let Some(i) = line.find("\u{0009}") {
            (line[..i].trim(), line[i..].trim())
        } else if let Some(i) = line.find(" ") {
            (line[..i].trim(), line[i..].trim())
        } else {
            (line, line)
        }
    };
    (a, b)
}
// call wrap for each element in the list
pub fn wrap_list(lines: &Vec<String>, w: u16) 
    -> Vec<(usize, String)> 
{
    let mut display: Vec<(usize, String)> = vec![];
    for (i, l) in lines.iter().enumerate() {
        let v = wrap(l, w);
        for s in v.iter() {
            display.push((i, s.to_string()));
        }
    }
    display
}
// wrap text in terminal
pub fn wrap(line: &str, screenwidth: u16) -> Vec<String> {
    let width = usize::from(screenwidth);
    let length = line.len();
    let mut wrapped: Vec<String> = vec![];
    // assume slice bounds
    let mut start = 0;
    let mut end = width;
    while end < length {
        start = line.ceil_char_boundary(start);
        end = line.floor_char_boundary(end);
        let longest = &line[start..end];
        // try to break line at a space
        match longest.rsplit_once(' ') {
            // there is a space to break on
            Some((a, b)) => {
                let shortest = match a.len() {
                    0 => b,
                    _ => a,
                };
                wrapped.push(String::from(shortest.trim()));
                start += shortest.len();
                end = start + width;
            }
            // there is no space to break on
            None => {
                wrapped.push(String::from(longest.trim()));
                start = end;
                end += width;
            }
        }
    }
    // add the remaining text
    if start < length {
        start = line.floor_char_boundary(start);
        wrapped.push(String::from(line[start..].trim()));
    }
    wrapped
}
#[derive(Clone, Debug)]
pub struct Editor {
    dscr:   DataScreen,
    pcol:   PosCol,
    text:   String,
    color:  Color,
}
impl Editor {
    pub fn new(dscr: &DataScreen, source: &str, color: Color) -> Self {
        Self {
            color:  color,
            pcol:   PosCol::origin(&dscr.outer.x),
            text:   source.into(),
            dscr:   dscr.clone(),
        }
    }
    pub fn resize(&mut self, dscr: &DataScreen) {
//        self.pcol.resize(self.text.len(), &range);
        self.dscr = dscr.clone();
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout.queue(cursor::Hide)?;
        let text = {
            let (a, b) = self.pcol.data_range(
                    &self.dscr.get_x_rng().outer, 
                    self.text.len());
                &self.text[a..b]
        };
        stdout
            .queue(MoveTo(self.dscr.outer.x.start, 8))?
            .queue(SetForegroundColor(self.color))?
            .queue(Print(text))?
            .queue(MoveTo(self.dscr.outer.x.start, self.dscr.outer.y.end))?
            .queue(Print(format!(
                        "{:?} {:?}", 
                        self.pcol, 
                        self.pcol.data_idx(&self.dscr.outer.x))))?
            .queue(MoveTo(self.pcol.cursor, 8))?
            .queue(cursor::Show)?
            .flush()
    }
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
    pub fn move_left(&mut self, step: u16) -> bool {
        self.pcol.move_backward(&self.dscr.get_x_rng(), step)
    }
    pub fn move_right(&mut self, step: u16) -> bool {
        self.pcol.move_forward(&self.dscr.get_x_rng(), self.text.len(), step)
    }
    pub fn delete(&mut self) -> bool {
        let idx = self.pcol.data_idx(&self.dscr.outer.x);
        if idx >= self.text.len() || self.text.len() == 0 {
            return false
        }
        self.text.remove(idx);
        if self.pcol.cursor + 1 != self.dscr.outer.x.end {
            self.pcol.move_forward(&self.dscr.get_x_rng(), self.text.len(), 1);
        }
        true
    }
    pub fn backspace(&mut self) -> bool {
        let idx = self.pcol.data_idx(&self.dscr.outer.x);
        if idx == 0 {
            return false
        } 
        self.pcol.move_backward(&self.dscr.get_x_rng(), 1);
        self.text.remove(idx);
        if self.pcol.cursor + 1 != self.dscr.outer.x.end {
            self.pcol.move_forward(&self.dscr.get_x_rng(), self.text.len(), 1);
        }
        true
    }
    pub fn insert(&mut self, c: char) -> bool {
        let idx = self.pcol.data_idx(&self.dscr.outer.x) + 1;
        if idx >= self.text.len() || self.text.len() == 0 {
            self.text.push(c);
        } else {
            self.text.insert(idx, c);
        }
        self.pcol.move_forward(&self.dscr.get_x_rng(), self.text.len(), 1);
        true
    }
}
#[derive(Clone, Debug)]
pub struct ColoredText {
    pub color: Color,
    pub text:  String,
}
impl ColoredText {
    pub fn from_vec(vec: &Vec<&str>, color: Color) -> Vec<Self> {
        vec.iter().map(|s| Self::new(s, color)).collect()
    }
    pub fn new(text: &str, color: Color) -> Self {
        Self {
            color: color,
            text: text.into(),
        }
    }
    pub fn getcolor(&self) -> Color {
        self.color
    }
}
#[derive(Clone, Debug)]
pub struct Reader {
    dscr:       DataScreen,
    pos:        Pos,
    source:     Vec<ColoredText>,
    display:    Vec<(usize, String)>,
    bounds:     Vec<usize>,
} 
impl Reader {
    pub fn one_color(   dscr: &DataScreen, 
                        source: &Vec<String>, 
                        color: Color    ) -> Self 
    {
        let text = source.iter().map(|s| ColoredText::new(s, color));
        Self::new(dscr, &text.collect())
    }
    pub fn new(dscr: &DataScreen, colored_text: &Vec<ColoredText>) -> Self {
        let display = wrap_list(
            &colored_text.iter().map(|ct| ct.text.clone()).collect(),
            dscr.outer.x.len16());
        let bounds = display.iter().map(|(_, s)| s.len());
        Self {
            dscr:    dscr.clone(),
            pos:     Pos::origin(&dscr.outer),
            source:  colored_text.clone(),
            bounds:  bounds.collect(),
            display: display,
        }
    }
    pub fn resize(&mut self, dscr: &DataScreen) {
        self.dscr    = dscr.clone();
        self.display = wrap_list(
            &self.source.iter().map(|ct| ct.text.clone()).collect(),
            dscr.outer.x.len16());
//      self.pcol.resize(
//          self.display.len(), 
//          &Range::verticle(rect));
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout.queue(cursor::Hide)?;
        let rngs = self.pos.get_ranges(&self.dscr, &self.bounds);
        for (scr_idx, idx, start, end) in rngs {
            let (key, text) = &self.display[idx];
            stdout
                .queue(MoveTo(self.dscr.outer.x.start, scr_idx))?
                .queue(SetForegroundColor(self.source[*key].color))?
                .queue(Print(&text[start..end]))?;
        }
        stdout
                .queue(MoveTo(self.dscr.outer.x.start, self.dscr.outer.y.end - 1))?
                .queue(Print(format!(
                            "{:?}",
                            self.dscr)))?
                .queue(MoveTo(self.dscr.outer.x.start, self.dscr.outer.y.end))?
                .queue(Print(format!(
                            "{:?} {:?}",
                            self.pos,
                            self.pos.y.data_idx(&self.dscr.outer.y))))?
            .queue(MoveTo(self.pos.x.cursor, self.pos.y.cursor))?
            .queue(cursor::Show)?
            .flush()
    }
    pub fn move_left(&mut self, step: u16) -> bool {
        self.pos.move_left(&self.dscr, step)
    }
    pub fn move_right(&mut self, step: u16) -> bool {
        self.pos.move_right(&self.dscr, &self.bounds, step)
    }
    pub fn move_up(&mut self, step: u16) -> bool {
        self.pos.move_up(&self.dscr, &self.bounds, step)
    }
    pub fn move_down(&mut self, step: u16) -> bool {
        self.pos.move_down(&self.dscr, &self.bounds, step)
    }
    pub fn select(&self) -> (usize, &str) {
        let idx = {
            let y_rng = self.dscr.outer.y.clone();
            let y_col = self.pos.y.clone();
            let idx = y_col.data_idx(&y_rng);
            self.display[idx].0
        };
        (idx, &self.source[idx].text)
    }
} 
