// src/dlg.rs

use crate::{
    cfg::{Config},
    screen::{Screen},
    editor::{Editor},
    msg::{InputMsg},
};
use crossterm::{
    QueueableCommand, cursor,
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
    Text(Editor),
}
impl InputType {
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match self {
            InputType::Text(editor) => {
                match keycode {
                    KeyCode::Enter => {
                        Some(InputMsg::Text(editor.get_text()))
                    }
                    KeyCode::Left => {
                        editor.move_left(1).then_some(InputMsg::Default)
                    }
                    KeyCode::Right => {
                        editor.move_right(1).then_some(InputMsg::Default)
                    }
                    KeyCode::Delete => {
                        editor.delete().then_some(InputMsg::Default)
                    }
                    KeyCode::Backspace => {
                        editor.backspace().then_some(InputMsg::Default)
                    }
                    KeyCode::Char(c) => {
                        editor.insert(*c);
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
#[derive(Clone)]
pub struct Dialog {
    dscr:       Screen,
    prompt:     String,
    input_type: InputType,
} 
impl Dialog {
    pub fn text(dscr: &Screen, cfg: &Config, prompt: &str) -> Self {
        Self {
            dscr:       dscr.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Text(
                Editor::new(dscr, "", cfg.colors.get_dialog())),
        }
    }
    pub fn ack(dscr: &Screen, cfg: &Config, prompt: &str) -> Self {
        Self {
            dscr:       dscr.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Ack(cfg.keys.dialog.ack),
        }
    }
    pub fn ask(dscr: &Screen, cfg: &Config, prompt: &str ) -> Self {
        Self {
            dscr:       dscr.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Ask( cfg.keys.dialog.yes, 
                                        cfg.keys.dialog.no  ),
        }
    }
    pub fn view(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(
                    self.dscr.outer.x().start, 
                    self.dscr.outer.y().start + 4))?
            .queue(Print(&self.prompt))?;
        match &mut self.input_type {
            InputType::Ack(ack) => {
                stdout.queue(cursor::MoveTo(
                            self.dscr.outer.x().start, 
                            self.dscr.outer.y().start + 8))?
                    .queue(Print(&format!("|{}| acknowledge", ack)))?;
            }
            InputType::Ask(yes, no) => {
                stdout.queue(cursor::MoveTo(
                            self.dscr.outer.x().start, 
                            self.dscr.outer.y().start + 8))?
                    .queue(Print(&format!("|{}| yes |{}| no", yes, no)))?;
            }
            InputType::Text(editor) => {
                editor.update_view()?;
                editor.view(stdout)?;
            }
        }
        Ok(())
    }
    pub fn resize(&mut self, dscr: &Screen) {
        self.dscr = dscr.clone();
        match &mut self.input_type {
            InputType::Text(editor) => {
                editor.resize(&self.dscr)
            }
            _ => {}
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => Some(InputMsg::Cancel),
            _ => self.input_type.update(keycode)
        }
    }
}
