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
pub enum State {
    Repair, 
    Running, 
    Stopped,
}
#[derive(Clone, PartialEq, Debug)]
pub enum View {
    AddressBar(Vec<u8>), 
    Prompt(String),
    Message(String),
    Text(i8, i8),
}
#[derive(Clone, Debug)]
pub struct Model {
    pub text: Option<GemTextDoc>,
    pub current: Option<Url>,
    pub view: View,
    pub state: State,
} 
impl Model {
    pub fn init(_url: &Option<Url>) -> Self {
        let Some(url) = _url else {
            return Self {
                current: None,
                text: None,
                state: State::Running,
                view: View::Message("nothing to display".to_string()),
            }
        };
        Self::from_url(url)
    }
    fn from_url(url: &Url) -> Self {
        let Ok((response, content)) = util::get_data(&url) else {
            return Self {
                current: None,
                text: None,
                state: State::Repair,
                view: View::Message("could not get data".to_string()),
            }
        };
        let Ok(status) = Status::from_str(&response) else {
            return Self {
                current: None,
                text: None,
                state: State::Repair,
                view: View::Message("could not parse status".to_string()),
            }
        };
        let (view, text) = match status {
            Status::InputExpected(variant, msg) => {
                (View::Message(format!("input: {}", msg)), None)
            }
            Status::Success(variant, meta) => {
                if meta.starts_with("text/") {
                    (View::Text(0, 0), Some(GemTextDoc::new(content)))
                } else {
                    (View::Message(format!("Recieved nontext response: {}", meta)), None)
                }
            }
            Status::PermanentFailure(variant, meta) => {
                (View::Message(format!("Permanent Failure {:?}: {:?}", variant, meta)), None)
            }
            Status::Redirect(variant, new_url) => {
                (View::Prompt(format!("Redirect to {}?", new_url)), None)
            }
            Status::ClientCertificateRequired(variant, meta) => {
                (View::Prompt(format!("Certificate required: {}", meta)), None)
            }
            _ => {
                (View::Message(format!("status type not handled")), None)
            }
        };
        Self {
            view: view,
            text: text,
            state: State::Running,
            current: Some(url.clone()),
        }
    }
} 
impl Widget for &Model {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = match &self.view {
            View::Text(x, y) => {
                format!("cursor at: ({}, {})\n{:#?}", x, y, self.text)
            }
            View::Message(msg) => {
                format!("Message: {}", msg)
            }
            View::Prompt(msg) => {
                format!("Prompt: {}", msg)
            }
            View::AddressBar(v) => {
                format!("Address Bar: {}", String::from_utf8_lossy(v))
            }
        };
        buf.set_string(area.x, area.y, text, Style::default());
    }
}
pub fn update(model: Model, msg: Message) -> Model {
    let mut m = model.clone();
    match msg {
        Message::Stop => { 
            m.state = State::Stopped;
        }
        Message::Enter => {
            m.view = View::Message("you pressed enter".to_string());
        }
        Message::Escape => { 
            m.view = View::Message("you pressed escape".to_string());
        }
        Message::Code(c) => {
            if let View::Text(x, y) = m.view {
                match c {
                    constants::LEFT  => m.view = View::Text(x - 1, y),
                    constants::RIGHT => m.view = View::Text(x + 1, y), 
                    constants::UP    => m.view = View::Text(x, y - 1),
                    constants::DOWN  => m.view = View::Text(x, y + 1),
                    _ => {}
                }
            } else {
                m.view = View::Message(format!("you pressed {}", c)); 
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
