// src/dlg.rs

use crate::{
    cfg::{Config},
    screen::{Frame},
    pos::{Pos},
    text::{Editor},
    msg::{InputMsg},
};
use crossterm::{
    QueueableCommand,
    cursor::{MoveTo},
    style::{Print},
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
    pub prompt_frame:   Frame,
    pub input_frame:    Frame,
    pub prompt:         String,
    pub input_type:     InputType,
} 
impl Dialog {
    fn new(frame: &Frame, prompt: &str) -> Self {
        Self {
            prompt_frame:   frame.row(3),
            input_frame:    frame.row(6),
            prompt:         prompt.into(), 
            input_type:     InputType::Ack('a'),
        }
    }

    pub fn text(frame: &Frame, cfg: &Config, prompt: &str) -> Self {
        let mut dlg     = Self::new(frame, prompt);
        let pos         = Pos::origin(&dlg.input_frame.outer);
        let editor      = Editor::new("", cfg.colors.get_dialog());
        dlg.input_type  = InputType::Text(editor, pos);
        dlg
    }

    pub fn ack(frame: &Frame, cfg: &Config, prompt: &str) -> Self {
        let mut dlg     = Self::new(frame, prompt);
        dlg.input_type  = InputType::Ack(cfg.keys.dialog.ack);
        dlg
    }

    pub fn ask(frame: &Frame, cfg: &Config, prompt: &str ) -> Self {
        let mut dlg     = Self::new(frame, prompt);
        dlg.input_type  = InputType::Ask
            (cfg.keys.dialog.yes, cfg.keys.dialog.no);
        dlg
    }

    pub fn view(&self, writer: &mut impl Write) -> io::Result<()> {
        let mut prompt_page = self.prompt_frame.get_page();
        let mut input_page  = self.input_frame.get_page();
        (prompt_page.buf[0]).queue(Print(&self.prompt))?;
        match &self.input_type {
            InputType::Ack(ack) => {
                (input_page.buf[0])
                    .queue(Print(&format!("|{}| acknowledge", ack)))?;
                prompt_page.view(writer)?;
                input_page.view(writer)?;
            }
            InputType::Ask(yes, no) => {
                (input_page.buf[0])
                    .queue(Print(&format!("|{}| yes |{}| no", yes, no)))?;
                prompt_page.view(writer)?;
                input_page.view(writer)?;
            }
            InputType::Text(editor, pos) => {
                input_page = editor.get_page(&self.input_frame, &pos);
                prompt_page.view(writer)?;
                input_page.view(writer)?;
                writer.queue(MoveTo(pos.x.cursor, pos.y.cursor))?;
            }
        }
        Ok(())
    }

    pub fn resize(&mut self, frame: &Frame) {
        self.prompt_frame   = frame.row(3);
        self.input_frame    = frame.row(6);
    }

    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => Some(InputMsg::Cancel),
            _ => self.update_input(keycode)
        }
    }

    fn update_input(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match &mut self.input_type {
            InputType::Text(editor, pos) => {
                match keycode {
                    KeyCode::Enter => {
                        Some(InputMsg::Text(editor.txt.clone()))
                    }
                    KeyCode::Left => {
                        pos.move_left(&self.input_frame, 1)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Right => {
                        pos.move_right(&self.input_frame, editor, 1)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Delete => {
                        editor.delete(&self.input_frame, pos)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Backspace => {
                        editor.backspace(&self.input_frame, pos)
                            .then_some(InputMsg::Default)
                    }
                    KeyCode::Char(c) => {
                        editor.insert(&self.input_frame, pos, *c);
                        Some(InputMsg::Default)
                    }
                    _ => None
                }
            }
            InputType::Ack(ack) => {
                match keycode {
                    KeyCode::Char(c) => {
                        (ack ==  c).then_some(InputMsg::Ack)
                    }
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
