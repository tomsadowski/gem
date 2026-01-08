// dialog

use crate::{
    util::{Rect},
    widget::{Pager, CursorText},
};
use crossterm::{
    QueueableCommand, cursor, style,
    event::{KeyCode},
};
use std::{
    io::{self, Stdout, Write},
};

#[derive(Clone, Debug)]
pub enum InputType {
    Choose {keys: Vec<char>, view: Pager},
    Text(CursorText),
}
impl InputType {
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match self {
            InputType::Text(cursortext) => {
                match keycode {
                    KeyCode::Enter => {
                        Some(InputMsg::Text(cursortext.get_text()))
                    }
                    KeyCode::Left => {
                        match cursortext.move_left(1) {
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Right => {
                        match cursortext.move_right(1) {
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Delete => {
                        match cursortext.delete() {
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Backspace => {
                        match cursortext.backspace() {
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Char(c) => {
                        cursortext.insert(*c);
                        Some(InputMsg::None)
                    }
                    _ => { 
                        None
                    }
                }
            }
            InputType::Choose {keys, ..} => {
                match keycode {
                    KeyCode::Char(c) => {
                        match keys.contains(&c) {
                            true => Some(InputMsg::Choose(*c)),
                            false => None,
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}
#[derive(Clone, Debug)]
pub enum InputMsg {
    None,
    Cancel,
    Choose(char),
    Text(String),
}
#[derive(Clone, Debug)]
pub struct Dialog {
    rect:       Rect,
    prompt:     String,
    input_type: InputType,
}
impl Dialog {
    pub fn text(rect: &Rect, prompt: &str) -> Self {
        Self {
            rect:       rect.clone(),
            prompt:     String::from(prompt), 
            input_type: InputType::Text(CursorText::new(rect, "")),
        }
    }
    pub fn choose(rect: &Rect, prompt: &str, choose: Vec<(char, &str)>) 
        -> Self
    {
        let view_rect = Rect {  x: rect.x, 
                                y: rect.y + 8, 
                                w: rect.w, 
                                h: rect.h - 8   };
        let keys_vec = choose.iter().map(|(c, _)| *c).collect();
        let view_vec = choose.iter()
                .map(|(x, y)| format!("|{}|  {}", x, y)).collect();
        let pager    = Pager::white(&view_rect, &view_vec);
        Self {
            rect:       rect.clone(),
            prompt:     String::from(prompt), 
            input_type: InputType::Choose { keys: keys_vec, 
                                            view: pager   },
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x, self.rect.y + 4))?
            .queue(style::Print(self.prompt.as_str()))?;
        match &self.input_type {
            InputType::Choose {view, ..} => {
                view.view(stdout)
            }
            InputType::Text(cursortext) => {
                cursortext.view(self.rect.y + 8, stdout)
            }
        }
    }
    // No wrapping yet, so resize is straightforward
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        match &mut self.input_type {
            InputType::Choose {view, ..} => {
                view.resize(&self.rect)
            }
            InputType::Text(cursortext) => {
                cursortext.resize(&self.rect)
            }
        }
    }
    // Keycode has various meanings depending on the InputType.
    // The match statement might be moved to impl InputType
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => Some(InputMsg::Cancel),
            _ => self.input_type.update(keycode)
        }
    }
}
