// dialog

use crate::{
    util::{Rect},
    widget::{Selector, CursorText},
};
use crossterm::{
    QueueableCommand, cursor, style,
    event::{KeyCode},
};
use std::{
    io::{self, Stdout, Write},
};

#[derive(Clone, Debug)]
pub struct ChooseBox {
    pub src: Vec<(char, String)>,
    pub wid: Selector,
}
#[derive(Clone, Debug)]
pub enum InputType {
    Choose(ChooseBox),
    Text(CursorText),
}
impl InputType {
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match (self, keycode) {
            (InputType::Text(cursortext), KeyCode::Enter) => {
                Some(InputMsg::Text(cursortext.get_text()))
            }
            (InputType::Text(cursortext), KeyCode::Char(c)) => {
                cursortext.insert(*c);
                Some(InputMsg::None)
            }
            (InputType::Text(cursortext), KeyCode::Left) => {
                match cursortext.move_left(1) {
                    true => Some(InputMsg::None),
                    false => None,
                }
            }
            (InputType::Text(cursortext), KeyCode::Right) => {
                match cursortext.move_right(1) {
                    true => Some(InputMsg::None),
                    false => None,
                }
            }
            (InputType::Text(cursortext), KeyCode::Delete) => {
                match cursortext.delete() {
                    true => Some(InputMsg::None),
                    false => None,
                }
            }
            (InputType::Text(cursortext), KeyCode::Backspace) => {
                match cursortext.backspace() {
                    true => Some(InputMsg::None),
                    false => None,
                }
            }
            (InputType::Choose(t), KeyCode::Char(c)) => {
                let chars: Vec<char> = 
                    t.src.iter().map(|e| e.0).collect();
                match chars.contains(&c) {
                    true => Some(InputMsg::Choose(*c)),
                    false => None,
                }
            }
            _ => None,
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
            input_type: InputType::Text(CursorText::new(rect, "tet")),
        }
    }
    pub fn choose(rect: &Rect, prompt: &str, choose: Vec<(char, &str)>) 
        -> Self
    {
        let choose_vec = choose.iter()
                .map(|(c, s)| (*c, s.to_string()))
                .collect();
        let selector_vec = choose.iter()
                .map(|(x, y)| format!("|{}|  {}", x, y))
                .collect();
        let selector_rect = 
            Rect {x: rect.x, y: rect.y + 8, w: rect.w, h: rect.h - 8};
        let selector = Selector::white(&selector_rect, &selector_vec);
        let choose_box = ChooseBox {src: choose_vec, wid: selector};
        Self {
            rect:       rect.clone(),
            prompt:     String::from(prompt), 
            input_type: InputType::Choose(choose_box),
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x, self.rect.y + 4))?
            .queue(style::Print(self.prompt.as_str()))?;
        match &self.input_type {
            InputType::Choose(choosebox) => {
                choosebox.wid.view(stdout)
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
            InputType::Choose(choosebox) => {
                choosebox.wid.resize(&self.rect)
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
