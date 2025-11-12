#![allow(unused_variables)]
#![allow(dead_code)]

mod model;
mod gemini;
mod util;

//  use crate::model;
//  use crate::util;
//  use crate::gemini;

use url::Url;
use std::io::{self, stdout};
//  use crossterm::event;
//  use ratatui::{
//      Terminal,
//      backend::CrosstermBackend, 
//      crossterm::{
//          ExecutableCommand,
//          terminal::{
//              disable_raw_mode, enable_raw_mode, 
//              EnterAlternateScreen, LeaveAlternateScreen,
//          },
//      },
//  };

// Notes:
// both Link and Status impl FromStr
fn main() -> io::Result<()> {

    // enter alternate screen
//  enable_raw_mode()?;
//  stdout().execute(EnterAlternateScreen)?;

    // data init
    // TODO: init with fun stuff
    let url = Url::parse(gemini::INIT_LINK).ok();
    let model = model::Model::init(&url);
    println!("{:#?}", model);

//    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // main loop
//  while model.state != model::State::Stopped {
//      // display model
//      terminal.draw(|f| f.render_widget(&model, f.area()))?;

//      // update model with event message
//      if let Some(message) = handle_event(&model, event::read()?) {
//          model = update(model, message);
//      }
//  }

    // ui close
//  stdout().execute(LeaveAlternateScreen)?;
//  disable_raw_mode()?;
    Ok(())
}

