// main

#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod model;
mod display;
mod gemtext;
mod gemstatus;
mod constants;
mod util;

// *** BEGIN IMPORTS ***
use url::Url;
use std::io::{
    self, 
    stdout
};
use crossterm::event;
use ratatui::{
    prelude::*, 
    text::{
        Line,
        Span,
        Text
    },
    style::{
        Color, 
        Style, 
        Modifier,
    },
    widgets::{
        Paragraph,
        Wrap
    },
    Terminal,
    backend::CrosstermBackend, 
    crossterm::{
        ExecutableCommand,
        terminal::{
            disable_raw_mode, enable_raw_mode, 
            EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
};
// *** END IMPORTS ***


fn main() -> io::Result<()> {

    // enter alternate screen
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    // data init
    let url          = Url::parse(constants::INIT_LINK).ok();
    let line_styles  = display::LineStyles {
        heading3:  Style::default(),
        heading2:  Style::default(),
        heading1:  Style::default(),
        link:      Style::default(),
        quote:     Style::default(),
        preformat: Style::default(),
        text:      Style::default(),
    };
    let mut display  = 
        display::DisplayModel::new(model::Model::init(&url), line_styles);
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // main loop
    while !display.source.quit {

        // display model
        terminal.draw(|f| f.render_widget(&display, f.area()))?;

        // update model with event message
        if let Some(message) = model::handle_event(event::read()?) {
            display.source = model::update(display.source, message);
        }
    }

    // ui close
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

