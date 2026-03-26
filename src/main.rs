// src/main.rs

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]

mod text;
mod screen;
mod widget;
mod usr;

use crate::{
  text::{TextPlane},
  screen::{Rect},
  widget::{TextBox},
};
use crossterm::{
  QueueableCommand,
  style::{
    Color, SetForegroundColor, 
    SetBackgroundColor, Print, ResetColor
  },
  cursor::{self, MoveTo},
  terminal::{self, Clear, ClearType},
  event::{self, Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use std::{
  fs, env,
  io::{self, stdout, Write},
};

fn main() -> io::Result<()> {
  let args: Vec<String> = env::args().collect();
  let path = args.get(1).unwrap(); 
  let text = fs::read_to_string(path).unwrap();

  terminal::enable_raw_mode()?;
  let (w, h) = terminal::size()?;
  let mut stdout = stdout();
  stdout
    .queue(terminal::EnterAlternateScreen)?
    .queue(terminal::DisableLineWrap)?;

  let mut ui = App::init(&text, w, h);
  ui.view(&mut stdout)?;

  while !ui.quit {
    if ui.update(event::read()?) {
      ui.view(&mut stdout)?;
    }
  }
  terminal::disable_raw_mode()?;
  stdout
    .queue(terminal::LeaveAlternateScreen)?
    .queue(terminal::EnableLineWrap)?
    .flush()
}

pub struct App {
  pub page:  TextBox,
  pub clear: bool,
  pub quit:  bool,
} 
impl App {
  pub fn init(text: &str, w: u16, h: u16) -> Self {
    Self {
      page: TextBox::new(text, w, h),
      quit: false, 
      clear: false
    }
  }
  pub fn view<W: Write>(&self, writer: &mut W) -> io::Result<()> { 
    if self.clear {
      writer.queue(Clear(ClearType::All))?;
    }
    self.page.view(writer)?;
    writer.flush()
  }
  pub fn update(&mut self, event: Event) -> bool {
    self.clear = false;
    match event {
      Event::Key(
        KeyEvent {
          modifiers: KeyModifiers::CONTROL,
          code: KeyCode::Char('c'),
          kind: KeyEventKind::Press, ..
        }
      ) => {
        self.quit = true;
        self.clear = true;
        true
      }
      Event::Resize(w, h) => {
        self.page.resize(w, h);
        self.clear = true;
        true
      }
      Event::Key(
        KeyEvent {
          code: kc, 
          kind: KeyEventKind::Press, ..
        }
      ) => {
        match kc {
          KeyCode::Left => {
            self.page.left()
          },
          KeyCode::Down => {
            self.page.down()
          },
          KeyCode::Up => {
            self.page.up()
          },
          KeyCode::Right => {
            self.page.right()
          },
          KeyCode::Backspace => {
            self.page.backspace()
          },
          KeyCode::Delete => {
            self.page.delete()
          },
          KeyCode::Char(c) => {
            self.page.insert(c)
          },
          _ => false,
        }
      }
      _ => false,
    }
  }
}
