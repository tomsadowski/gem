// ui

use crate::{
    config::{self, Config},
    util::{Rect},
    tab::{TabServer},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    style::{self, Color},
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use std::{
    io::{self, Stdout, Write},
};

// view currently in use
#[derive(Debug)]
pub enum View {
    Tab,
    Quit,
}
// coordinates activities between views
pub struct UI {
    rect:    Rect,
    bgcolor: Color,
    view:    View,
    config:  Config,
    tabs:    TabServer,
} 
impl UI {
    // start with View::Tab
    pub fn new(config: &Config, w: u16, h: u16) -> Self {
        let rect = Rect {x: 0, y: 0, w: w, h: h};
        Self {
            tabs:    TabServer::new(&rect, config),
            rect:    rect,
            config:  config.clone(),
            view:    View::Tab,
            bgcolor: config::getbackground(&config.colors),
        }
    }
    // resize all views, maybe do this in parallel?
    fn resize(&mut self, w: u16, h: u16) {
        self.rect = Rect {x: 0, y: 0, w: w, h: h};
        self.tabs.resize(&self.rect);
    }
    // display the current view
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(style::SetBackgroundColor(self.bgcolor))?;
        match &self.view {
            View::Tab => self.tabs.view(stdout),
            _ => Ok(()),
        }?;
        stdout.flush()
    }
    // Resize and Control-C is handled here, 
    // otherwise delegate to current view
    pub fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Resize(w, h) => {
                self.resize(w, h); 
                true
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press, ..
            }) => {
                self.view = View::Quit;
                true
            }
            Event::Key(KeyEvent {
                code: keycode, 
                kind: KeyEventKind::Press, ..
            }) => 
                match &self.view {
                    View::Tab => self.tabs.update(&keycode),
                    _ => false,
                }
            _ => false,
        }
    }
    // no need to derive PartialEq for View
    pub fn quit(&self) -> bool {
        match self.view {
            View::Quit => true,
            _ => false,
        }
    }
} 
