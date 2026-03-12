// src/dlg.rs

use crate::{
  usr::{User},
  screen::{Frame},
  pos::{Pos},
  text::{self, Text, Editor},
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
  Ack(KeyCode),
  Ask(KeyCode, KeyCode),
  Text(Editor, Pos),
}

#[derive(Clone)]
pub struct Dialog {
  pub prompt_frame: Frame,
  pub prompt_text:  Text,
  pub input_frame:  Frame,
  pub input_type:   InputType,
} 

impl Dialog {

  fn new(frm: &Frame, prompt_text: &str) -> Self {

    Self {
      prompt_frame: frm.row(3),
      input_frame: frm.row(6),
      prompt_text: prompt_text.into(), 
      input_type: InputType::Ack(KeyCode::Enter),
    }
  }

  pub fn text(frm: &Frame, usr: &User, prompt_text: &str)
    -> Self 
  {
    let mut dlg = Self::new(frm, prompt_text);
    let pos = dlg.input_frame.pos();
    let editor = Editor::new(
      &dlg.input_frame, "", usr.colors.dialog);

    dlg.input_type = InputType::Text(editor, pos);
    dlg
  }

  pub fn ack(frm: &Frame, usr: &User, prompt_text: &str) 
    -> Self 
  {
    let mut dlg = Self::new(frm, prompt_text);

    dlg.input_type = 
      InputType::Ack(usr.keys.ack);
    dlg
  }

  pub fn ask(frame: &Frame, 
             usr: &User, 
             prompt_text: &str) 
    -> Self 
  {
    let mut dlg = Self::new(frame, prompt_text);

    dlg.input_type = InputType::Ask
      (usr.keys.yes, usr.keys.no);
    dlg
  }

  pub fn view<W>(&self, writer: &mut W) -> io::Result<()> 
  where W: Write
  {
    self.prompt_text
      .write_frame(&self.prompt_frame, writer)?;

    match &self.input_type {
      InputType::Ack(ack) => {
        Text::from(
          format!("|{}| acknowledge", ack).as_str())
          .write_frame(&self.input_frame, writer)?;
      }

      InputType::Ask(yes, no) => {
        Text::from(
          format!("|{}| yes |{}| no", yes, no).as_str())
          .write_frame(&self.input_frame, writer)?;
      }

      InputType::Text(editor, pos) => {
        editor.write_frame(&self.input_frame, writer)?;
        writer.queue(MoveTo(pos.x.cursor, pos.y.cursor))?;
      }
    }
    Ok(())
  }

  pub fn resize(&mut self, frame: &Frame) {
    self.prompt_frame = frame.row(3);
    self.input_frame  = frame.row(6);
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
              .move_left(&self.input_frame, 1)
              .then_some(InputMsg::Default)
          }

          KeyCode::Right => {
            editor
              .move_right(&self.input_frame, 1)
              .then_some(InputMsg::Default)
          }

          KeyCode::Delete => {
            editor
              .delete(&self.input_frame, pos)
              .then_some(InputMsg::Default)
          }

          KeyCode::Backspace => {
            editor
              .backspace(&self.input_frame, pos)
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
