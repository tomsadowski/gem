// src/app.rs

use crate::{
  text::{TextPlane},
  screen::{Rect},
  widget::{TextBox},
  usr::{User, Layout, Keys},
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
  io::{self, stdout, Stdout, Write, Error},
};

pub struct App {
  pub usr:      User,
  pub page:     TextBox,
  pub clear:    bool,
  pub stdout:   Stdout,
  pub quit:     bool,
} 
impl App {
  pub fn init(usr_path: &str) -> Result<Self, Error> {
    terminal::enable_raw_mode()?;
    let usr_text = fs::read_to_string(usr_path).unwrap_or_default();
    let usr = User::try_from_string(&usr_text).unwrap_or_default();
    let text = fs::read_to_string(&usr.init_path).unwrap_or_default();
    let (w, h) = terminal::size()?;
    let mut rect = Rect::new(w, h);
    rect.crop_x(usr.layout.x_page);
    rect.crop_y(usr.layout.y_page);
    let mut text_box = TextBox::new(&text, &rect);
    text_box.fg = usr.layout.text;
    text_box.bg = usr.layout.background;
    let mut app = Self {
      usr,
      stdout:   stdout(),
      page:     text_box,
      quit:     false, 
      clear:    false
    };
    app.stdout
      .queue(terminal::EnterAlternateScreen)?
      .queue(terminal::DisableLineWrap)?;
    Ok(app)
  }
  pub fn run(&mut self) -> io::Result<()> {
    while !self.quit {
      if self.update(event::read()?) {
        if self.clear {
          self.stdout.queue(Clear(ClearType::All))?;
        }
        self.page.view(&mut self.stdout)?;
        self.stdout.flush()?;
      }
    }
    terminal::disable_raw_mode()?;
    self.stdout
      .queue(terminal::LeaveAlternateScreen)?
      .queue(terminal::EnableLineWrap)?
      .flush()
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
        if kc == self.usr.keys.left {
          self.page.left()
        } else if kc == self.usr.keys.down {
          self.page.down()
        } else if kc == self.usr.keys.up {
          self.page.up()
        } else if kc == self.usr.keys.right {
          self.page.right()
        } else if kc == KeyCode::Backspace {
          self.page.backspace()
        } else if kc == KeyCode::Delete {
          self.page.delete()
        } else {
          //self.page.insert(c)
          false
        }
      }
      _ => false,
    }
  }
}
