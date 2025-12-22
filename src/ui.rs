// gem/src/ui
// joins backend and frontend
use crate::{
    config::{Config},
    gemini::{GemTextData},
    widget::{GetColors, Rect},
    tabs::TabMgr};
use crossterm::{
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
    style::{Colors, Color}};
use std::{
    io::{self, Stdout, Write}};

// view currently in use
#[derive(Debug)]
pub enum View {
    Tab,
    Quit,
}
// coordinates activities between views
pub struct UI {
    rect: Rect,
    view: View,
    config: Config,
    tabs: TabMgr,
} 
impl UI {
    // start with View::Tab
    pub fn new(config: &Config, w: u16, h: u16) -> Self {
        let rect = Rect::new(0, 0, w, h);
        Self {
            tabs: TabMgr::new(&rect, config),
            rect: rect,
            config: config.clone(),
            view: View::Tab,
        }
    }
    // resize all views, maybe do this in parallel?
    fn resize(&mut self, w: u16, h: u16) {
        self.rect = Rect::new(0, 0, w, h);
        self.tabs.resize(&self.rect);
    }
    // display the current view
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
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
                kind: KeyEventKind::Press, 
                ..
            }) => {
                self.view = View::Quit;
                true
            }
            Event::Key(KeyEvent {
                code: keycode, 
                kind: KeyEventKind::Press, 
                ..
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
impl GetColors for GemTextData {
    fn getcolors(&self) -> Colors {
        match self {
            Self::HeadingOne => Colors::new(
                Color::Rgb {r: 225, g: 180, b: 105},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::HeadingTwo => Colors::new(
                Color::Rgb {r: 225, g: 180, b: 105},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::HeadingThree => Colors::new(
                Color::Rgb {r: 225, g: 180, b: 105},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::Text => Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::Quote => Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::ListItem => Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::PreFormat => Colors::new(
                Color::Rgb {r: 80 , g: 180, b: 80 },
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::Link(_) => Colors::new(
                Color::Rgb {r: 105, g: 180, b: 225},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
        } 
    }
}
