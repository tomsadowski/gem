// src/app.rs

use crate::{
  usr::{User},
  screen::{Page, Rect},
  msg::{Focus, ViewMsg},
  text::{Doc, Text},
  tab::Tab,
};
use crossterm::{
  QueueableCommand, cursor,
  terminal::{Clear, ClearType},
  style::{Color},
  event::{
    Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers,
  }
};
use std::{
  fs,
  io::{self, Write}
};

pub struct App {
  pub hdr:      Doc,
  pub tabs:     Vec<Tab>,
  pub hdr_page: Page,
  pub tab_page: Page,
  pub idx:      usize,
  pub usr_path: String,
  pub usr:      User,
  pub focus:    Focus,
  pub clr_scr:  bool,
  pub quit:     bool,
} 
impl App {
  pub fn init(path: &str, w: u16, h: u16) -> Self {

    let usr = Self::load_config(path);
    let (hdr_page, tab_page) = 
      usr.get_layout(w, h);

    let mut app = Self {
      usr_path: path.into(),
      quit:     false, 
      focus:    Focus::Tab,
      idx:      0,  
      hdr:      Doc::default(),
      clr_scr:  false,
      tabs:     vec![
        Tab::init(&tab_page, &usr.init_url, &usr)],
      hdr_page, 
      tab_page,
      usr,
    };

    app.update_hdr_text();
    app
  }

  pub fn view(&self, writer: &mut impl Write) 
    -> io::Result<()> 
  { 
    writer.queue(cursor::Hide)?;

    if self.clr_scr {
      writer.queue(Clear(ClearType::All))?;
    }

    self.hdr.view(&self.hdr_page, writer)?;
    self.tabs[self.idx].view(writer)?;

    writer
      .queue(cursor::Show)?
      .flush()
  }

  pub fn update(&mut self, event: Event) -> bool {

    self.clr_scr = false;

    match event {
      Event::Key(
        KeyEvent {
          modifiers: KeyModifiers::CONTROL,
          code: KeyCode::Char('c'),
          kind: KeyEventKind::Press, ..
        }
      ) => {
        self.quit = true;
        true
      }

      Event::Resize(w, h) => {
        self.resize(w, h);
        self.clr_scr = true;
        true
      }

      Event::Key(
        KeyEvent {
          code: kc, 
          kind: KeyEventKind::Press, ..
        }
      ) => {

        let response = match &self.focus {
          Focus::Global => 
            self.update_global(&kc),
          Focus::Tab => 
            self.tabs[self.idx]
              .update(&self.usr, &kc),
        }; 

        if let Some(msg) = response { 
          self.update_from_view_msg(msg);
          self.update_hdr_text();
          true
        } else {
          false
        }
      }

      _ => 
        false,
    }
  }

  fn update_from_view_msg(&mut self, msg: ViewMsg) {

    match msg {

      ViewMsg::Global => {
        self.focus = Focus::Global;
      }

      ViewMsg::ReloadUser => {
        self.update_usr(
          Self::load_config(&self.usr_path));
        self.clr_scr = true;
      }

      ViewMsg::NewUser(s) => {
        self.usr_path = s;
        self.update_usr(
          Self::load_config(&self.usr_path));
        self.clr_scr = true;
      }

      ViewMsg::Go(url) => {
        let tab = Tab::init(
          &self.tab_page, &url, &self.usr);

        self.tabs.push(tab);
        self.idx = self.tabs.len() - 1;
        self.clr_scr = true;
      }

      ViewMsg::DeleteMe => {
        if self.tabs.len() > 1 {
          self.tabs.remove(self.idx);
          self.idx = self.tabs.len() - 1;
          self.clr_scr = true;
        }
      }

      ViewMsg::CycleLeft => {
        if self.idx == 0 {
          self.idx = self.tabs.len() - 1;
        } else {
          self.idx -= 1;
        }
        self.clr_scr = true;
      }

      ViewMsg::CycleRight => {
        if self.idx == self.tabs.len() - 1 {
          self.idx = 0;
        } else {
          self.idx += 1;
        }
        self.clr_scr = true;
      }

      _ => {}
    }
  }

  fn resize(&mut self, w: u16, h: u16) {

    let (hdr_page, tab_page) = 
      self.usr.get_layout(w, h);

    self.hdr_page = hdr_page;
    self.tab_page = tab_page;

    for t in self.tabs.iter_mut() {
      t.resize(&self.tab_page);
    }
    self.update_hdr_text();
  }

  fn update_hdr_text(&mut self) {

    let fg = self.usr.layout.banner.unwrap_or(Color::White);
    let bg = self.usr.layout.background.unwrap_or(Color::Black);

    let info = format!("{}/{}: {}", 
                       self.idx + 1, 
                       self.tabs.len(), 
                       &self.tabs[self.idx].name);

    let line = &String::from("-")
      .repeat(self.hdr_page.text.w);

    self.hdr = Doc::new(
      vec![
        Text::from(info.as_str()).fg(fg).bg(bg),
        Text::from(line.as_str()).fg(fg).bg(bg), 
      ],
      &self.hdr_page
    );
  }

  // return default config if error
  fn load_config(path: &str) -> User {
    fs::read_to_string(path)
      .map_err(|e| e.to_string())
      .and_then(|txt| User::parse(&txt)).unwrap()
  }

  fn update_usr(&mut self, usr: User) {

    self.usr = usr;

    for t in self.tabs.iter_mut() {
      t.update_usr(&self.usr);
    }
  }

  fn update_global(&mut self, keycode: &KeyCode) 
    -> Option<ViewMsg> 
  {
    if keycode == &self.usr.keys.cancel {
      self.focus = Focus::Tab;
      Some(ViewMsg::Default)

    } else if keycode == &self.usr.keys.tab_view {
      self.focus = Focus::Tab;
      Some(ViewMsg::Default)

    } else if keycode == &self.usr.keys.load_usr {
      self.focus = Focus::Tab;
      Some(ViewMsg::ReloadUser)

    } else {
      None
    }
  }
}
