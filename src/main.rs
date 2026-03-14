// src/main.rs

#![allow(dead_code)]
//#![allow(unused_imports)]
#![allow(unused_variables)]

mod util;
mod gem;
mod dlg;
mod usr;
mod usr_keys;
mod usr_layout;
mod usr_text;
mod text;
mod tab;
mod app;
mod pos;
mod screen;
mod msg;

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
  let mut ui = App::init("gem.toml", w, h);

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
