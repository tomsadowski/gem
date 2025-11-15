// model

use url::Url;
use std::str::FromStr;
use crate::{
    util, 
    gemtext::GemTextDoc,
    gemstatus::Status,
    constants,
};
use ratatui::prelude::*;
use crossterm::{
    event::{self, 
        KeyModifiers, 
        KeyEvent, 
        Event, 
        KeyEventKind, 
        KeyCode},
};


#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    Code(char),
    Enter,
    Escape,
    Stop,
}
#[derive(Clone, PartialEq, Debug)]
pub enum Dialog {
    AddressBar(Vec<u8>), 
    Prompt(String, Vec<u8>),
    Message(String),
}
#[derive(Clone, Debug)]
pub enum Address {
    Url(Url), 
    String(String),
}
#[derive(Clone, Debug)]
pub enum Text {
    GemText(GemTextDoc), 
    String(String),
}
#[derive(Clone, Debug)]
pub struct Model {
    pub dialog: Option<Dialog>,
    pub address: Address,
    pub text: Text,
    pub quit: bool,
    pub x: usize,
    pub y: usize,
} 
impl Dialog {
    pub fn init_from_response(status: Status) -> Option<Self> {
        match status {
            Status::InputExpected(variant, msg) => {
                Some(
                    Self::Prompt(
                        format!("input: {}", msg), 
                        vec![]
                    )
                )
            }
            Status::Success(variant, meta) => {
                if meta.starts_with("text/") {
                    None
                } else {
                    Some(
                        Self::Prompt(
                            format!("Download nontext type: {}?", meta), 
                            vec![]
                        )
                    )
                }
            }
            Status::TemporaryFailure(variant, meta) => {
                None
            }
            Status::PermanentFailure(variant, meta) => {
                None
            }
            Status::Redirect(variant, new_url) => {
                Some(
                    Self::Prompt(
                        format!("Redirect to: {}?", new_url), 
                        vec![]
                    )
                )
            }
            Status::ClientCertificateRequired(variant, meta) => {
                Some(
                    Self::Prompt(
                        format!("Certificate required: {}", meta),
                        vec![]
                    )
                )
            }
        }
    }
}
impl Text {
    pub fn init_from_response(status: Status, content: String) -> Self {
        match status {
            Status::InputExpected(variant, msg) => {
                Self::String(content)
            }
            Status::Success(variant, meta) => {
                if meta.starts_with("text/") {
                    Self::GemText(GemTextDoc::new(content))
                } else {
                    Self::String(String::from("no text"))
                }
            }
            Status::TemporaryFailure(variant, meta) => {
                Self::String(format!("Temporary Failure {:?}: {:?}", variant, meta))
            }
            Status::PermanentFailure(variant, meta) => {
                Self::String(format!("Permanent Failure {:?}: {:?}", variant, meta))
            }
            Status::Redirect(variant, new_url) => {
                Self::String(format!("Redirect to: {}?", new_url))
            }
            Status::ClientCertificateRequired(variant, meta) => {
                Self::String(format!("Certificate required: {}", meta))
            }
        }
    }
}
impl Model {
    pub fn init(_url: &Option<Url>) -> Self {
        let Some(url) = _url else {
            return Self {
                address: Address::String(String::from("")),
                text: Text::String(String::from("welcome")),
                dialog: None,
                quit: false,
                x: 0,
                y: 0,
            }
        };
        let Ok((header, content)) = util::get_data(&url) else {
            return Self {
                address: Address::Url(url.clone()),
                text: Text::String(String::from("no get data")),
                dialog: None,
                quit: false,
                x: 0,
                y: 0,
            }
        };
        let Ok(status) = Status::from_str(&header) else {
            return Self {
                address: Address::Url(url.clone()),
                text: Text::String(String::from("could not parse status")),
                dialog: None,
                quit: false,
                x: 0,
                y: 0,
            }
        };
        Self {
            address: Address::Url(url.clone()),
            text: Text::init_from_response(status.clone(), content),
            dialog: Dialog::init_from_response(status),
            quit: false,
            x: 0,
            y: 0,
        }
    }
} 
impl Widget for &Model {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = format!("{:#?}", self);
        buf.set_string(area.x, area.y, text, Style::default());
    }
}
pub fn update(model: Model, msg: Message) -> Model {
    let mut m = model.clone();
    match msg {
        Message::Stop => { 
            m.quit = true;
        }
        Message::Enter => {
            m.dialog = None;
        }
        Message::Escape => { 
            m.dialog = None;
        }
        Message::Code(c) => {
            if let None = m.dialog {
                match c {
                    constants::LEFT  => {},
                    constants::RIGHT => {}, 
                    constants::UP    => {},
                    constants::DOWN  => {},
                    _ => {}
                }
            } else {
                m.dialog = Some(Dialog::Message(format!("you pressed {}", c))); 
            }
        }
    }
    m
}
pub fn handle_event(event: event::Event) -> Option<Message> {
    let Event::Key(keyevent) = event 
        else {return None};
    match keyevent {
        KeyEvent {
            code: KeyCode::Char('c'),
            kind: KeyEventKind::Press,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            Some(Message::Stop)
        }
        KeyEvent {
            code: KeyCode::Enter,
            kind: KeyEventKind::Press,
            ..
        } => {
            Some(Message::Enter)
        }
        KeyEvent {
            code: KeyCode::Esc,
            kind: KeyEventKind::Press,
            ..
        } => {
            Some(Message::Escape)
        }
        KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            ..
        } => {
            Some(Message::Code(c))
        }
        _ => None
    }
}

//  fn follow_link(&mut self, link: &str) {
//      let next_url = match &self.current {
//          Some(current) => {
//              // for relative url
//              current.join(link).expect("Not a URL")
//          },
//          None => Url::parse(link).expect("Not a URL")
//      };
//      self.visit_url(&next_url)
//  }
//  fn reload_page(model: &mut Model) {
//      // Get current URL from history and revisit it without modifying history
//      let Some(url) = model.cur().clone() else {return};
//      match util::get_data(url) {
//          Ok((meta, new_content)) => {
//              // handle meta header
//              let response = handle_response_status
//                  (model, url.clone(), meta, new_content);
//          }
//          Err(msg) => {
//          }
//      }
//  }
//  fn visit_url(model: &mut Model, url: Url) {
//      match util::get_data(&url) {
//          Ok((meta, new_content)) => {
//              model.history.push(url.clone());

//              // handle meta header
//              if let Some(response) = 
//                  handle_response_status(model, url, meta, new_content)
//              {
//                  model.content = response;
//              }
//          }
//          Err(msg) => {
//          }
//      }
//  }
//  fn set_title(s: &mut Cursive, text: &str) {
//      let mut container = match s.find_name::<Dialog>("container") {
//          Some(view) => view,
//          None => panic!("Can't find container view."),
//      };
//      container.set_title(text);
//  }
//  fn follow_line(s: &mut Cursive, line: &str) {
//      let parsed = json::parse(line);
//      if let Ok(data) = parsed {
//          if link::is_gemini(&data) {
//              let current_url = history::get_current_url().unwrap();
//              let next_url = current_url
//                  .join(&data["url"].to_string())
//                  .expect("Not a URL");
//              visit_url(s, &next_url)
//          } 
//          else {
//              open::that(data["url"].to_string()).unwrap();
//          }
//      }
//  }
//  fn prompt_for_url(s: &mut Cursive) {
//      s.add_layer(
//          Dialog::new()
//              .title("Enter URL")
//              .padding(Margins::lrtb(1, 1, 1, 0))
//              .content(EditView::new().on_submit(goto_url).fixed_width(20))
//              .with_name("url_popup"),
//      );
//  }
//  fn prompt_for_answer(s: &mut Cursive, url: Url, message: String) {
//      s.add_layer(
//          Dialog::new()
//              .title(message)
//              .padding(Margins::lrtb(1, 1, 1, 0))
//              .content(
//                  EditView::new()
//                      .on_submit(move |s, response| {
//                          let link = format!("{}?{}", url.to_string(), response);
//                          s.pop_layer();
//                          follow_link(s, &link);
//                      })
//                      .fixed_width(60),
//              )
//              .with_name("url_query"),
//      );
//  }
//  fn prompt_for_secret_answer(s: &mut Cursive, url: Url, message: String) {
//      s.add_layer(
//          Dialog::new()
//              .title(message)
//              .padding(Margins::lrtb(1, 1, 1, 0))
//              .content(
//                  EditView::new().secret().on_submit(
//                      move |s, response| {
//                          let link = format!("{}?{}", url.to_string(), response);
//                          s.pop_layer();
//                          follow_link(s, &link);
//                      }
//                  )
//                  .fixed_width(60),
//              )
//              .with_name("url_query"),
//      );
//  }
