// model

use crate::{
    util, 
    view::TextView,
    view::Dialog,
    view::Action,
    gemini::Status,
    gemini::GemTextLine,
    gemini::GemTextData,
    gemini::Scheme,
};
use std::io::{
    self, Write, Stdout
};
use crossterm::event::{
    Event, KeyEvent, KeyEventKind, KeyCode
};

const LEFT:  char = 'e';
const DOWN:  char = 'i';
const UP:    char = 'o';
const RIGHT: char = 'n';
const QUIT:  char = 'q';

const URL:   char = 'g';

#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    Code(char),
    Resize(u16, u16),
    Enter,
    Escape,
    Stop,
}
impl Message {
    // given a relevant Event, return some Message
    pub fn from_event(event: Event) -> Option<Message> {
        match event {
            Event::Key(keyevent) => Self::from_key_event(keyevent),
            Event::Resize(y, x)  => Some(Message::Resize(y, x)),
            _                    => None,
        }
    }

    // given a relevant KeyEvent, return some Message
    fn from_key_event(keyevent: KeyEvent) -> Option<Message> {
        match keyevent {
            KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            } => {
                Some(Message::Code(c))
            }
            _ => 
                None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Model<'a, 'b> {
    quit:   bool,
    dialog: Option<Dialog>,
    text:   TextView<'a, 'b>,
    url:    Option<url::Url>,
}
impl<'a: 'b, 'b> Model<'a, 'b> {
    pub fn new() -> Self {
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        match self.dialog {
            Some(dialog) => dialog.view(stdout),
            None => self.text.view(stdout),
        }
        Ok(())
    }

    // return new model based on old model and message
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Stop => { 
                self.quit = true;
            }
            Message::Escape => { 
                self.dialog = None;
            }
            Message::Enter => {
                if let Some(dialog) = self.dialog.clone() {
                    self = respond_to_dialog(self, dialog);
                }
                else { 
                    let data = self.text.get_text_under_cursor().0;
                    self.dialog = query_gemtext_data(data);
                }
            }
            Message::Resize(_y, _x) => {
            }
            Message::Code(c) => {
                if let None = self.dialog {
                    match c {
                        UP   => self.text.move_cursor_up(),
                        DOWN => self.text.move_cursor_down(),
                        QUIT => self.quit = true,
                        _ => {}
                    }
                } 
            }
        }
    }
    
    fn respond_to_dialog(&mut self, dialog: Dialog) {
        match dialog.action {
            Action::FollowLink(url) => {
                if let Ok((header, content)) = util::get_data(&url) {
                    if let Ok(status) = Status::from_str(&header) {
                        self.text = 
                            self.text.update_from_response(status, content);
                    }
                }
            },
            _ => {}
        }
        self.dialog = None;
    }
}
