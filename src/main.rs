// gem/src/main

#![allow(dead_code)]
#![allow(unused_imports)]

mod gemini;     // backend
mod widget;     // frontend
mod common;     // used by backend and frontend
mod ui;         // uses backend and frontend
mod config;     // uses backend and frontend

use crossterm::{
    QueueableCommand, terminal, event,
};
use std::{
    io::{self, stdout, Write},
    fs,
};

fn main() -> io::Result<()> {
    let (w, h) = terminal::size()?;
    let mut ui = ui::UI::new("gem.toml", w, h);
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?;
    ui.view(&stdout)?;

    // main loop
    while !ui.is_quit() {
        if ui.update(event::read()?) {
            ui.view(&stdout)?;
        }
    }

    // clean up
    terminal::disable_raw_mode()?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()
}
