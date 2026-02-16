// src/main.rs

#![allow(dead_code)]
#![allow(unused_imports)]

mod util;
mod gem;
mod dlg;
mod cfg;
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
    let (w, h) = terminal::size()?;
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?;
    let mut ui = App::new("gem.toml", w, h);
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
