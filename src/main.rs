// src/main.rs

#![allow(dead_code)]
//#![allow(unused_imports)]
#![allow(unused_variables)]

mod util;
mod gem;
mod pos;
mod page;
mod usr;
mod msg;
mod app;
mod tab;
mod text;
mod dlg;

use crate::{
  app::App,
};
use crossterm::{
  QueueableCommand, terminal, event,
};
use std::{
  io::{self, stdout, Write}
};

fn main() -> io::Result<()> {
  terminal::enable_raw_mode()?;

  let mut stdout = stdout();

  stdout
    .queue(terminal::EnterAlternateScreen)?
    .queue(terminal::DisableLineWrap)?;

  let (w, h) = terminal::size()?;
  let mut ui = App::init(".gemsettings", w, h);

  ui.view(&mut stdout)?;

  while !ui.quit {
    if ui.update(event::read()?) {
      ui.view(&mut stdout)?;
    }
  }

  terminal::disable_raw_mode()?;
  stdout.queue(terminal::LeaveAlternateScreen)?;
  stdout.flush()
}
