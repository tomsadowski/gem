// src/dlg.rs

use crate::{
    cfg::{Config},
    screen::{Frame},
    pos::{Pos},
    text::{self, white_line, Editor},
    msg::{InputMsg},
};
use crossterm::{
    QueueableCommand,
    cursor::{MoveTo},
    style::{Print, Color},
    event::{KeyCode},
};
use std::{
    io::{self, Write}
};

#[derive(Clone)]
pub enum InputType {
    Ack(char),
    Ask(char, char),
    Text(Editor, Pos),
}
#[derive(Clone)]
pub struct Dialog {
    pub p_frm:  Frame,
    pub i_frm:  Frame,
    pub p_txt:  String,
    pub i_type: InputType,
} 
impl Dialog {
    fn new(frm: &Frame, p_txt: &str) -> Self {
        Self {
            p_frm:  frm.row(3),
            i_frm:  frm.row(6),
            p_txt:  p_txt.into(), 
            i_type: InputType::Ack('a'),
        }
    }

    pub fn text(frm: &Frame, cfg: &Config, p_txt: &str) -> Self {
        let mut dlg = Self::new(frm, p_txt);
        let pos     = dlg.i_frm.pos();
        let editor  = Editor::new(&dlg.i_frm, "", cfg.colors.get_dialog());
        dlg.i_type  = InputType::Text(editor, pos);
        dlg
    }

    pub fn ack(frm: &Frame, cfg: &Config, p_txt: &str) -> Self {
        let mut dlg = Self::new(frm, p_txt);
        dlg.i_type  = InputType::Ack(cfg.keys.dialog.ack);
        dlg
    }

    pub fn ask(frame: &Frame, cfg: &Config, p_txt: &str ) -> Self {
        let mut dlg = Self::new(frame, p_txt);
        dlg.i_type  = InputType::Ask
            (cfg.keys.dialog.yes, cfg.keys.dialog.no);
        dlg
    }

    pub fn view<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        white_line(&self.p_txt, &self.p_frm, writer)?;
        match &self.i_type {
            InputType::Ack(ack) => {
                white_line(
                    &format!("|{}| acknowledge", ack),
                    &self.i_frm, 
                    writer)?;
            }
            InputType::Ask(yes, no) => {
                white_line(
                    &format!("|{}| yes |{}| no", yes, no),
                    &self.i_frm, 
                    writer)?;
            }
            InputType::Text(editor, pos) => {
                editor.view(&self.i_frm, &pos, writer)?;
                writer.queue(MoveTo(pos.x.cursor, pos.y.cursor))?;
            }
        }
        Ok(())
    }

    pub fn resize(&mut self, frame: &Frame) {
        self.p_frm  = frame.row(3);
        self.i_frm  = frame.row(6);
    }

    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => Some(InputMsg::Cancel),
            _ => self.update_input(keycode)
        }
    }

    fn update_input(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match &mut self.i_type {
            InputType::Text(editor, pos) => {
                match keycode {
                    KeyCode::Enter => {
                        Some(InputMsg::Text(editor.txt.clone()))
                    }
                    KeyCode::Left => {
                        editor
                            .move_left(&self.i_frm, 1)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Right => {
                        editor
                            .move_right(&self.i_frm, 1)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Delete => {
                        editor
                            .delete(&self.i_frm, pos)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Backspace => {
                        editor
                            .backspace(&self.i_frm, pos)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Char(c) => {
                        editor.insert(&self.i_frm, pos, *c);
                        Some(InputMsg::Default)
                    }
                    _ => None
                }
            }
            InputType::Ack(ack) => {
                match keycode {
                    KeyCode::Char(c) => 
                        (ack ==  c).then_some(InputMsg::Ack),
                    _ => None,
                }
            }
            InputType::Ask(yes, no) => {
                match keycode {
                    KeyCode::Char(c) => {
                        if yes ==  c {
                            Some(InputMsg::Yes)
                        } else if no == c {
                            Some(InputMsg::No)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}
