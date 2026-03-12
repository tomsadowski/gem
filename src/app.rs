// src/app.rs

use crate::{
  usr::{self, User},
  screen::{Frame, Rect},
  msg::{Focus, ViewMsg},
  text::{Doc, Text},
  tab::Tab,
};
use crossterm::{
  QueueableCommand, cursor,
  terminal::{Clear, ClearType},
  style::{Color},
  event::{
    Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers}
};
use std::{
  fs,
  io::{self, Write}
};

pub struct App {
  pub hdr:          Doc,
  pub tabs:         Vec<Tab>,
  pub hdr_frame:    Frame,
  pub tab_frame:    Frame,
  pub idx:          usize,
  pub usr_path:     String,
  pub usr:          User,
  pub focus:        Focus,
  pub clr_scr:      bool,
  pub quit:         bool,
} 
impl App {
  pub fn new(path: &str, w: u16, h: u16) -> Self {

    let usr = Self::load_config(path);
    let (hdr_frame, tab_frame) = 
      Self::get_layout(w, h, &usr);

    let mut app = Self {
      usr_path: path.into(),
      quit:     false, 
      focus:    Focus::Tab,
      idx:      0,  
      hdr:      Doc::default(),
      clr_scr:  false,
      tabs:     vec![
        Tab::init(&tab_frame, &usr.init_url, &usr)],
      hdr_frame, 
      tab_frame,
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

    self.hdr.view(&self.hdr_frame, writer)?;
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

  // called infrequently, construct many things
  // based on screensize and usr
  fn get_layout(w: u16, h: u16, usr: &User) 
    -> (Frame, Frame) 
  {
    let (hdr_rect, tab_rect) = {

      let rect = Rect::new(w, h)
        .crop_x(usr.format.x_margin)
        .crop_y(usr.format.y_margin);

      (rect.crop_south(h - 2), rect.crop_north(2))

    };

    let hdr = Frame::new(&hdr_rect, 0, 0);
    let tab = Frame::new(&tab_rect, 
                         usr.scroll_at, 
                         usr.scroll_at);

    (hdr, tab)
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
          &self.tab_frame, &url, &self.usr);

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

    let (hdr_frame, tab_frame) = 
      Self::get_layout(w, h, &self.usr);

    self.hdr_frame = hdr_frame;
    self.tab_frame = tab_frame;

    for t in self.tabs.iter_mut() {
      t.resize(&self.tab_frame);
    }
    self.update_hdr_text();
  }

  fn update_hdr_text(&mut self) {

    let fg = self.usr.colors.banner;
    let bg = self.usr.colors.background;

    let info = format!("{}/{}: {}", 
                       self.idx + 1, 
                       self.tabs.len(), 
                       &self.tabs[self.idx].name);

    let line = &String::from("-")
      .repeat(self.hdr_frame.outer.w);

    self.hdr = Doc::new(
      vec![
        Text::from(info.as_str()).fg(fg).bg(bg),
        Text::from(line.as_str()).fg(fg).bg(bg), 
      ],
      &self.hdr_frame
    );
  }

  // return default config if error
  fn load_config(path: &str) -> User {
    fs::read_to_string(path)
      .ok()
      .map(|txt| User::parse(&txt))
      .unwrap_or_default()
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
