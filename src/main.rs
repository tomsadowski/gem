#![allow(unused_variables)]
#![allow(dead_code)]

mod model;
mod gemtext;
mod gemstatus;
mod constants;
mod util;

//  use crate::model;
//  use crate::util;
//  use crate::gemini;

use url::Url;
use std::io::{
    self, 
    stdout
};
    use crossterm::event;
    use ratatui::{
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

fn main() -> io::Result<()> {

    // enter alternate screen
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    // data init
    // TODO: init with fun stuff
    let url = Url::parse(constants::INIT_LINK).ok();
    let mut model = model::Model::init(&url);
//    println!("{:#?}", model);

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // main loop
    while model.state != model::State::Stopped {
        // display model
        terminal.draw(|f| f.render_widget(&model, f.area()))?;

        // update model with event message
        if let Some(message) = model::handle_event(event::read()?) {
            model = model::update(model, message);
        }
    }

    // ui close
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

