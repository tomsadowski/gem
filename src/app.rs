// src/app.rs

use crate::{
  text::{TextPlane},
  screen::{Rect},
  widget::{TextBox},
  usr::{self, User, Style, Keys},
};
use crossterm::{
  QueueableCommand,
  terminal::{self, Clear, ClearType},
  cursor::SetCursorStyle,
  event::{self, Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use std::{
  fs, env,
  str::FromStr,
  io::{self, Write, Stdout},
};

pub struct App {
  pub usr:   User,
  pub page:  TextBox,
  pub clear: bool,
  pub quit:  bool,
} 
impl App {
  pub fn run(path: &str, stdout: &mut Stdout) -> io::Result<()> {
    // initialize app
    let mut app = {
      let usr_text = 
        fs::read_to_string(path)
          .unwrap();
      let usr = 
        User::from_str(&usr_text)
          .unwrap();
      let text = 
        fs::read_to_string(&usr.init_url)
          .unwrap();
      let (w, h) = terminal::size()?;
      let mut rect = Rect::new(w, h);
      rect.crop_x(usr.style.x_page_margin);
      rect.crop_y(usr.style.y_page_margin);
      let mut text_box = TextBox::new(
        &text, 
        &rect, 
        usr.style.x_text_margin, 
        usr.style.y_text_margin);
      text_box.fg = usr.style.fg;
      text_box.bg = usr.style.bg;
      Self {
        usr,
        page:  text_box,
        quit:  false, 
        clear: false
      }
    };
    // register keystrokes 
    terminal::enable_raw_mode()?;
    // handle line wrapping manually
    stdout
      .queue(terminal::EnterAlternateScreen)?
      .queue(terminal::DisableLineWrap)?
      .queue(SetCursorStyle::SteadyBar)?
      ;
    // initial display
    app.view(stdout)?;
    // main loop
    while !app.quit {
      if app.update(event::read()?) {
        app.view(stdout)?;
      }
    }
    // return terminal to normal state
    stdout
      .queue(terminal::LeaveAlternateScreen)?
      .queue(terminal::EnableLineWrap)?
      .queue(SetCursorStyle::DefaultUserShape)?
      .flush()?;
    terminal::disable_raw_mode()
  }
  fn view(&mut self, stdout: &mut Stdout) -> io::Result<()> {
    if self.clear {
      stdout.queue(Clear(ClearType::All))?;
    }
    self.page.view(stdout)?;
    stdout.flush()
  }
  fn update(&mut self, event: Event) -> bool {
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
        let mut page_rect = Rect::new(w, h);
        page_rect.crop_x(self.usr.style.x_page_margin);
        page_rect.crop_y(self.usr.style.y_page_margin);
        self.page.resize(&page_rect,
          self.usr.style.x_text_margin, 
          self.usr.style.y_text_margin);
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
          self.page.left(1)
        } else if kc == self.usr.keys.down {
          self.page.down(1)
        } else if kc == self.usr.keys.up {
          self.page.up(1)
        } else if kc == self.usr.keys.right {
          self.page.right(1)
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
