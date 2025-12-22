// gem/src/dialog
use crate::{
    widget::{Rect}};
use crossterm::{
    QueueableCommand, cursor, style,
    event::KeyCode};
use std::{
    io::{self, Stdout}};

#[derive(Clone, Debug)]
pub enum DialogMsg<T> {
    None,
    Cancel,
    Submit(T),
}
#[derive(Clone, Debug)]
pub enum InputType {
    None,
    Choose((char, Vec<(char, String)>)),
    Input(String),
}
#[derive(Clone, Debug)]
pub struct Dialog<T> {
    rect: Rect,
    prompt: String,
    pub action: T,
    pub input: InputType,
}
impl<T: Clone + std::fmt::Debug> Dialog<T> {
    pub fn new(rect: &Rect, action: T, input: InputType, prompt: &str) -> Self 
    {
        Self {
            rect: rect.clone(),
            action: action,
            input: input,
            prompt: String::from(prompt), 
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x + 2, self.rect.y + 2))?
            .queue(style::Print(self.prompt.as_str()))?
            .queue(cursor::MoveTo(self.rect.x + 2, self.rect.y + 4))?
            .queue(style::Print(format!("{:?}", self.input)))?;
        Ok(())
    }
    // No wrapping yet, so resize is straightforward
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
    }
    // Keycode has various meanings depending on the InputType.
    // The match statement might be moved to impl InputType
    pub fn update(&mut self, keycode: &KeyCode) -> Option<DialogMsg<T>> {
        match (&mut self.input, keycode) {
            // Pressing Escape always cancels
            (_, KeyCode::Esc) => {
                Some(DialogMsg::Cancel)
            }
            // Pressing Enter in a choosebox means nothing
            (InputType::Choose(_), KeyCode::Enter) => {
                Some(DialogMsg::None)
            }
            // Otherwise, pressing Enter means Submit
            (_, KeyCode::Enter) => {
                Some(DialogMsg::Submit(self.action.clone()))
            }
            // Backspace works in inputbox
            (InputType::Input(v), KeyCode::Backspace) => {
                v.pop();
                Some(DialogMsg::None)
            }
            // Typing works in inputbox
            (InputType::Input(v), KeyCode::Char(c)) => {
                v.push(*c);
                Some(DialogMsg::None)
            }
            // Check for meaning in choosebox
            (InputType::Choose(t), KeyCode::Char(c)) => {
                let chars: Vec<char> = t.1.iter().map(|e| e.0).collect();
                match chars.contains(&c) {
                    true => {
                        t.0 = *c;
                        Some(DialogMsg::Submit(self.action.clone()))
                    }
                    false => None,
                }
            }
            _ => None,
        }
    }
}
