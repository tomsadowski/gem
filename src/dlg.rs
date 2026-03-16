// src/dlg.rs

use crate::{
  page::{Page},
  pos::{Pos},
  text::{Text, Editor},
  msg::{InputMsg},
};
use crossterm::{
  QueueableCommand,
  cursor::{MoveTo},
  event::{KeyCode},
};
use std::{
  io::{self, Write}
};

#[derive(Clone)]
pub enum InputType {
  Ack(KeyCode),
  Ask(KeyCode, KeyCode),
  Text(Editor, Pos),
}

#[derive(Clone)]
pub struct Dialog {
  pub prompt_page: Page,
  pub prompt_text: Text,
  pub input_page:  Page,
  pub input_type:  InputType,
} 
impl Dialog {

  pub fn new(page: &Page, text: &str) -> Self {
    Self {
      prompt_page:  page.row(3),
      input_page:   page.row(6),
      prompt_text:  text.into(), 
      input_type:   InputType::Ack(KeyCode::Enter),
    }
  }

  pub fn view<W>(&self, writer: &mut W) -> io::Result<()> 
  where W: Write
  {
    self.prompt_text
      .write_page(&self.prompt_page, writer)?;

    match &self.input_type {
      InputType::Ack(ack) => {
        Text::from(
          format!("|{}| acknowledge", ack).as_str())
          .write_page(&self.input_page, writer)?;
      }

      InputType::Ask(yes, no) => {
        Text::from(
          format!("|{}| yes |{}| no", yes, no).as_str())
          .write_page(&self.input_page, writer)?;
      }

      InputType::Text(editor, pos) => {
        editor.write_page(&self.input_page, writer)?;
        writer.queue(MoveTo(pos.x.cursor, pos.y.cursor))?;
      }
    }
    Ok(())
  }

  pub fn resize(&mut self, page: &Page) {
    self.prompt_page = page.row(3);
    self.input_page  = page.row(6);
  }

  pub fn update(&mut self, keycode: &KeyCode) 
    -> Option<InputMsg> 
  {
    match keycode {
      KeyCode::Esc => 
        Some(InputMsg::Cancel),

      _ => 
        self.update_input(keycode)
    }
  }

  fn update_input(&mut self, keycode: &KeyCode) 
    -> Option<InputMsg> 
  {
    match &mut self.input_type {
      InputType::Text(editor, pos) => {
        match keycode {
          KeyCode::Enter => {
            Some(InputMsg::Text(editor.txt.clone()))
          }

          KeyCode::Left => {
            editor
              .move_left(&self.input_page, 1)
              .then_some(InputMsg::Default)
          }

          KeyCode::Right => {
            editor
              .move_right(&self.input_page, 1)
              .then_some(InputMsg::Default)
          }

          KeyCode::Delete => {
            editor
              .delete(&self.input_page, pos)
              .then_some(InputMsg::Default)
          }

          KeyCode::Backspace => {
            editor
              .backspace(&self.input_page, pos)
              .then_some(InputMsg::Default)
          }

          KeyCode::Char(c) => {
            editor.insert(&self.input_page, pos, *c);
            Some(InputMsg::Default)
          }

          _ => None
        }
      }

      InputType::Ack(ack) => {
        (ack ==  keycode).then_some(InputMsg::Ack)
      }

      InputType::Ask(yes, no) => {
        if yes ==  keycode {
            Some(InputMsg::Yes)
        } else if no == keycode {
            Some(InputMsg::No)
        } else {
            None
        }
      }
    }
  }
}
