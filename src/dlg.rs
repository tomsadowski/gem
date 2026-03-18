// src/dlg.rs

use crate::{
  page::{Page},
  pos::{Pos},
  text::{Text, Editor, TextWidget},
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
  Text(Editor),
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

      InputType::Text(editor) => {
        editor.view(writer)?;
      }
    }
    Ok(())
  }


  pub fn resize(&mut self, page: &Page) {
    self.prompt_page = page.row(3);
    self.input_page  = page.row(6);
    if let InputType::Text(editor) = &mut self.input_type {
      editor.resize(page);
    }
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
      InputType::Text(editor) => {
        match keycode {
          KeyCode::Enter => {
            Some(InputMsg::Text(editor.text.clone()))
          }

          KeyCode::Left => {
            editor
              .move_left(1)
              .then_some(InputMsg::Default)
          }

          KeyCode::Right => {
            editor
              .move_right(1)
              .then_some(InputMsg::Default)
          }

          KeyCode::Delete => {
            editor
              .delete()
              .then_some(InputMsg::Default)
          }

          KeyCode::Backspace => {
            editor
              .backspace()
              .then_some(InputMsg::Default)
          }

          KeyCode::Char(c) => {
            editor.insert(*c);
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
