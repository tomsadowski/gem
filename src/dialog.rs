// dialog

use crate::{
    geometry::{Rect},
    widget::{Selector},
};
use crossterm::{
    QueueableCommand, cursor, style,
    event::{KeyCode},
};
use std::{
    io::{self, Stdout},
};

#[derive(Clone, Debug)]
pub enum InputType {
    None,
    Choose(Vec<(char, String)>),
    Input(String),
}
impl InputType {
    // shortcut to create inputbox
    pub fn input() -> Self {
        Self::Input(String::from(""))
    }
    // shortcut to create choosebox
    pub fn choose(vec: Vec<(char, &str)>) -> Self {
        Self::Choose(
            vec
            .iter()
            .map(|(c, s)| (*c, s.to_string()))
            .collect())
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match (self, keycode) {
            // Pressing Enter in a choosebox means nothing
            (InputType::None, KeyCode::Enter) => {
                Some(InputMsg::Confirm)
            }
            (InputType::Input(s), KeyCode::Enter) => {
                Some(InputMsg::Input(s.to_string()))
            }
            (_, KeyCode::Enter) => {
                Some(InputMsg::None)
            }
            // Pressing Escape always cancels
            // Backspace works in inputbox
            (InputType::Input(v), KeyCode::Backspace) => {
                v.pop();
                Some(InputMsg::None)
            }
            // Typing works in inputbox
            (InputType::Input(v), KeyCode::Char(c)) => {
                v.push(*c);
                Some(InputMsg::None)
            }
            // Check for meaning in choosebox
            (InputType::Choose(t), KeyCode::Char(c)) => {
                let chars: Vec<char> = t.iter().map(|e| e.0).collect();
                match chars.contains(&c) {
                    true => {
                        Some(InputMsg::Choose(*c))
                    }
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
    Confirm,
    Choose(char),
    Input(String),
}
#[derive(Clone, Debug)]
pub struct InputBox {
    pub selector: Selector,
    pub inputtype: InputType,
}
impl InputBox {
    pub fn new(rect: &Rect, inputtype: InputType) -> Self {
        let selector = match &inputtype {
            InputType::Choose(v) => {
                let m = v
                    .iter()
                    .map(|(x, y)| format!("|{}|  {}", x, y))
                    .collect();
                Selector::white(rect, &m)
            }
            InputType::Input(s) => {
                Selector::white(rect, &vec![s.clone()])
            }
            _ => Selector::white(rect, &vec![]),
        };
        Self {
            selector: selector,
            inputtype: inputtype,
        }
    }
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        self.selector.view(stdout)?;
        Ok(())
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        self.inputtype.update(keycode)
    }
}
#[derive(Clone, Debug)]
pub enum DialogMsg<T> {
    None,
    Cancel,
    Submit(T, InputMsg),
}
#[derive(Clone, Debug)]
pub struct Dialog<T> {
    rect: Rect,
    prompt: String,
    pub action: T,
    pub inputbox: InputBox,
}
impl<T: Clone + std::fmt::Debug> Dialog<T> {
    pub fn new(rect: &Rect, 
               action: T, 
               input: InputType, 
               prompt: &str) -> Self 
    {
        Self {
            rect: rect.clone(),
            action: action,
            inputbox: InputBox::new(rect, input),
            prompt: String::from(prompt), 
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x + 2, self.rect.y + 2))?
            .queue(style::Print(self.prompt.as_str()))?
            .queue(cursor::MoveTo(self.rect.x + 2, self.rect.y + 4))?;
        self.inputbox.view(stdout)?;
        Ok(())
    }
    // No wrapping yet, so resize is straightforward
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
    }
    // Keycode has various meanings depending on the InputType.
    // The match statement might be moved to impl InputType
    pub fn update(&mut self, keycode: &KeyCode) -> Option<DialogMsg<T>> {
        match keycode {
            KeyCode::Esc => 
                Some(DialogMsg::Cancel),
            _ => 
                match self.inputbox.update(keycode) {
                    None => None,
                    Some(InputMsg::None) =>
                        Some(DialogMsg::None),
                    Some(submit) => 
                        Some(DialogMsg::Submit(
                                self.action.clone(), submit)),
            }
        }
    }
}
