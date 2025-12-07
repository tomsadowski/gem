// main

#![allow(dead_code)]
#![allow(unused_imports)]

mod util;
mod gemini;
mod text;
mod textview;
mod dialog;
mod model;

use crate::{
    text::{
        GemTextBlock,
    },
    model::{
        Message, Model,
    },
};
use url::Url;
use crossterm::{
    QueueableCommand, terminal, cursor, event
};
use std::io::{
    self, stdout, Write
};

// elm paradigm
fn main() -> io::Result<()> {
    // init
    terminal::enable_raw_mode()?;
    let     url    = Url::parse("gemini://geminiprotocol.net/").ok();
    let mut model  = Model::new(&url, terminal::size()?);
    let mut stdout = stdout();

    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?
        .queue(cursor::Show)?;
    stdout.flush()?;

    while !model.quit() {
        // display model
        model.view(&stdout)?;

        // update model with event message.
        // note that calling `event::read()` blocks until
        // an event is encountered.
        if let Some(msg) = Message::from_event(event::read()?) {
            model.update(msg);
        }
    }

    // clean up
    terminal::disable_raw_mode()?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;
    Ok(())
}
