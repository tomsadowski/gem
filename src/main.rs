// gem/src/main

#![allow(dead_code)]

mod gemini;    // frontend agnostic
mod util;      // frontend agnostic
mod widget;    // backend agnostic
mod dialog;    // backend agnostic
mod ui;        // joins backend and frontend
mod tabserver; // joins backend and frontend
mod tab;       // joins backend and frontend
mod config;    // keybindings, visuals

use crate::{
    ui::UI,
    config::Config,
};
use crossterm::{
    QueueableCommand, terminal, cursor, event,
};
use std::{
    io::{self, stdout, Write},
    fs,
};

fn main() -> io::Result<()> {
    let configtext = fs::read_to_string("gem.toml").unwrap();
    let config = Config::new(configtext.as_str());
    
    let (w, h) = terminal::size()?;

    let mut ui = UI::new(&config, w, h);
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?
        .queue(cursor::Show)?;
    stdout.flush()?;
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
