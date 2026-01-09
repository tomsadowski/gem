// gem/src/main

#![allow(dead_code)]
#![allow(unused_imports)]

mod gemini;     // backend
mod widget;     // frontend
mod common;     // used by backend and frontend
mod ui;         // uses backend and frontend
mod config;     // uses backend and frontend

use crossterm::{
    QueueableCommand, terminal, cursor, event,
};
use std::{
    io::{self, stdout, Write},
    fs,
};

fn main() -> io::Result<()> {
    let configtext = fs::read_to_string("gem.toml").unwrap();
    let config = config::Config::new(configtext.as_str());
    let (w, h) = terminal::size()?;
    let mut ui = ui::UI::new(&config, w, h);
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?
        .queue(cursor::Show)?;
    ui.view(&stdout)?;
    // main loop
    while !ui.quit() {
        if ui.update(event::read()?) {
            ui.view(&stdout)?;
        }
    }
    // clean up
    terminal::disable_raw_mode()?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;
    Ok(())
}
