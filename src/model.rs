// model

use url::Url;
use std::str::FromStr;
use crate::{
    util, 
    gemini::{GemTextDoc, Status}
};
use crossterm::{
    event::{KeyCode},
};
use ratatui::{
    prelude::*,
    layout::{Direction},
};
#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    Switch(View), 
    Move(Direction, i8), 
    Edit(KeyCode), 
    GoToUrl(Url), 
    Stop,
}
#[derive(Clone, PartialEq, Debug)]
pub enum State {
    Running, 
    Stopped
}
#[derive(Clone, PartialEq, Debug)]
pub enum View {
    AddressBar, 
    Prompt(String),
    Dialogue(String),
    Message(String),
    Text,
}
#[derive(Clone, Debug)]
pub struct Model {
    pub status: Option<Status>,
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
                status: None,
                text: None,
                state: State::Running,
                view: View::Message("nothin to show ya".to_string()),
            }
        };
        let data = util::get_data(&url);
        let (response, text) = 
            match data {
                Ok((r, c)) => (r, Some(GemTextDoc::new(c))),
                Err(r)     => (r, None),
            };
        Self {
            current: Some(url.clone()),
            status: Status::from_str(&response).ok(),
            text: text,
            state: State::Running,
            view: View::Text,
        }
    }
//  fn go_to_url(&mut self, url: &Url) {
//      if let Ok((response, content)) = util::get_data(&url) {
//          let status = Status::from_str(&response).unwrap();
//          match status {
//              Status::Success(meta) => {
//                  if meta.starts_with("text/") {
//                      // display text files.
//                      self.view = View::Text;
//                      self.text = Some(GemTextDoc::new(content));
//                  } else {
//                      // download and try to open the rest.
//                      self.text = None;
//                  }
//              }
//              Status::Gone(meta) => {
//                  self.text = None;
//                  self.view = 
//                      View::Message(format!("gone :( {}", meta));
//              }
//              Status::RedirectTemporary(new_url)
//              | Status::RedirectPermanent(new_url) => {
//                  self.text = None;
//              }
//              Status::TransientCertificateRequired(_meta)
//              | Status::AuthorisedCertificatedRequired(_meta) => {
//                  self.view = 
//                      View::Prompt(format!("certificate required: {}", meta));
//                  self.text = None;
//              }
//              Status::Input(message) => {
//                  self.text = None;
//              }
//              Status::Secret(message) => {
//                  self.text = None;
//              }
//              other_status => {
//                  self.text = None;
//              }
//          }
//      }
//  }
} 
impl Widget for &Model {
    fn render(self, area: Rect, buf: &mut Buffer) {
    }
}


//  use ratatui::prelude::*;
//  use crossterm::{
//      event::{self, 
//          KeyModifiers, 
//          KeyEvent, 
//          Event, 
//          KeyEventKind, 
//          KeyCode},
//  };

//  fn process_response(&mut self, status: Status, content: String) {
//      match status {
//          Status::Success(meta) => {
//              if meta.starts_with("text/") {
//                  // display text files.
//                  self.focus = View::Content;
//              } else {
//                  // download and try to open the rest.
//                  util::download(content);
//              }
//          }
//          Status::Gone(_meta) => {
//              self.focus = View::Prompt("Sorry page is gone.".to_string());
//          }
//          Status::RedirectTemporary(new_url)
//          | Status::RedirectPermanent(new_url) => {
//              follow_link(&new_url);
//              self.content = None;
//          }
//          Status::TransientCertificateRequired(_meta)
//          | Status::AuthorisedCertificatedRequired(_meta) => {
//              s.add_layer(Dialog::info(
//                  "You need a valid certificate to access this page.",
//              ));
//              None
//          }
//          Status::Input(message) => {
//              prompt_for_answer(s, url_copy, message);
//              None
//          }
//          Status::Secret(message) => {
//              prompt_for_secret_answer(s, url_copy, message);
//              None
//          }
//          other_status => {
//              s.add_layer(Dialog::info(format!("ERROR: {:?}", other_status)));
//              None
//          }
//      }
//  }

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



//  pub fn update(model: Model, msg: Message) -> Model {
//      let mut m = model.clone();
//      match msg {
//          Message::GoToUrl(url) => {
//              visit_url(&mut m, url); m
//          }
//          Message::Stop => { 
//              m.state = State::Stopped; m 
//          }
//          Message::Switch(view) => { 
//              m.focus = view; m 
//          }
//          Message::Move(direction, steps) => {
//              match direction {
//                  Direction::Horizontal => {
//                      match model.focus {
//                          View::Content => { 
//                             // m.content.move_horizontal(steps); 
//                              m 
//                          }
//                          View::AddressBar => { 
//                             // m.address_bar.move_horizontal(steps); 
//                             m 
//                          }
//                          _ => {
//                              model
//                          }
//                      }
//                  }
//                  Direction::Vertical => {
//                      match model.focus {
//                          View::Content => { 
//                            //  m.content.move_vertical(steps); 
//                              m 
//                          _ => {
//                              model
//                          }
//                      }
//                  }
//              }
//          }
//          Message::Edit(keycode) => {
//              match model.focus {
//                  View::AddressBar => { 
//                     // m.address_bar.edit(keycode); 
//                      m 
//                  }
//                  _ => {
//                      model
//                  }
//              }
//          }
//      }
//  }
//  pub fn handle_event(model: &Model, event: event::Event) -> Option<Message> {
//      let Event::Key(keyevent) = event else {return None};
//      match keyevent {
//          KeyEvent {
//              code: KeyCode::Char('c'),
//              kind: KeyEventKind::Press,
//              modifiers: KeyModifiers::CONTROL,
//              ..
//          } => {
//              Some(Message::Stop)
//          }
//          KeyEvent {
//              code: KeyCode::Enter,
//              kind: KeyEventKind::Press,
//              ..
//          } => {
//              Some(Message::GoToUrl(model.next.clone()))
//          }
//          KeyEvent {
//              code: KeyCode::Esc,
//              kind: KeyEventKind::Press,
//              ..
//          } => {
//              Some(Message::Switch(View::Content))
//          }
//          KeyEvent {
//              code: KeyCode::Char(c),
//              kind: KeyEventKind::Press,
//              ..
//          } => {
//              match c {
//                  util::DOWN if model.focus != View::AddressBar => {
//                      Some(Message::Move(Direction::Vertical, -1))
//                  }
//                  util::UP if model.focus != View::AddressBar => {
//                      Some(Message::Move(Direction::Vertical, 1))
//                  }
//                  util::LEFT if model.focus != View::AddressBar => {
//                      Some(Message::Move(Direction::Horizontal, -1))
//                  }
//                  util::RIGHT if model.focus != View::AddressBar => {
//                      Some(Message::Move(Direction::Horizontal, 1))
//                  }
//                  _ => None
//              }
//          }
//          _ => None
//      }
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
