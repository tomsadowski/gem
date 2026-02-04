// widget

use crate::{
    common,
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
            pcol:   Pos::origin(&dscr.outer).get_x_col(),
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
            let (a, b) = screen::data_range(&self.dscr.get_x_rng().outer, 
                                            &self.pcol, 
                                            self.text.len());
                &self.text[a..b]
        };
        stdout
            .queue(MoveTo(self.dscr.outer.x, self.dscr.outer.y))?
            .queue(SetForegroundColor(self.color))?
            .queue(Print(text))?
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
        let idx = screen::data_idx(&self.dscr.get_x_rng().outer, &self.pcol);
        if idx == self.text.len() {
            return false
        }
        self.text.remove(idx);
//      self.pcol.resize(self.text.len(), &self.range);
//      let end = std::cmp::min(self.dscr.outer.)
//      if self.pcol.cursor + 1 != self.range.end() {
//          self.pcol.move_forward(&self.dscr.get_x_rng(), self.text.len(), 1);
//      }
        true
    }
    pub fn backspace(&mut self) -> bool {
//      if self.pcol.is_start() {
//          return false
//      } 
//      self.pcol.backward(1);
//      self.text.remove(self.pcol.get_index());
//      self.pcol.resize(self.text.len(), &self.range);
//      if usize::from(self.pcol.get_pcol()) + 1 != self.range.end() {
//          self.pcol.forward(1);
//      }
        true
    }
    pub fn insert(&mut self, c: char) -> bool {
//      if self.pcol.get_index() + 1 == self.text.len() {
//          self.text.push(c);
//      } else {
//          self.text.insert(self.pcol.get_index(), c);
//      }
//      self.pcol.resize(self.text.len(), &self.range);
//      self.pcol.forward(1);
        true
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
        let display = common::wrap_list(
            &colored_text.iter().map(|ct| ct.text.clone()).collect(),
            dscr.outer.w);
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
        self.display = common::wrap_list(
            &self.source.iter().map(|ct| ct.text.clone()).collect(),
            dscr.outer.w);
//      self.pcol.resize(
//          self.display.len(), 
//          &Range::verticle(rect));
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout.queue(cursor::Hide)?;
        let (a, b) = self.pcol.get_display_range();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(MoveTo(  self.dscr.outer.x, 
                                self.pcol.get_screen_start() + j as u16))?
                .queue(SetForegroundColor(self.source[*i].color))?
                .queue(Print(text.as_str()))?;
        }
        stdout
            .queue(MoveTo(  self.rect.x, 
                            self.pcol.get_pcol()))?
            .queue(pcol::Show)?
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
            let y_rng = self.dscr.outer.get_y_rng();
            let y_col = self.pos.get_y_col();
            let idx = screen::data_idx(&y_rng, &y_col);
            self.display[idx].0
        };
        (idx, &self.source[idx].text)
    }
} 
