// src/tab.rs

use crate::{
  usr::{User},
  util::{Scheme},
  gem::{GemDoc, GemTag, Status},
  text::{Doc},
  screen::{Frame},
  msg::{ViewMsg, InputMsg},
  dlg::{Dialog},
};
use crossterm::{
  event::{KeyCode}
};
use std::{
  io::{self, Write}
};
use url::{Url};


pub struct Tab {
  pub frame:  Frame,
  pub name:   String,
  pub dlg:    Option<(ViewMsg, Dialog)>,
  pub gdoc:   Option<GemDoc>,
  pub ddoc:   Doc,
} 

impl Tab {
  pub fn init(frame: &Frame, url_str: &str, usr: &User) 
    -> Self 
  {
    let mut tab = Self {
      dlg:    None,
      gdoc:   None,
      ddoc:   Doc::default(), 
      frame:  frame.clone(),
      name:   url_str.into(),
    };

    tab.make_request(usr, url_str);
    tab
  }

  // resize ddoc and dialog
  pub fn resize(&mut self, frame: &Frame) {

    self.frame = frame.clone();
    self.ddoc.resize(frame);

    if let Some((_, d)) = &mut self.dlg {
      d.resize(frame);
    }
  }

  pub fn update(&mut self, usr: &User, kc: &KeyCode) 
    -> Option<ViewMsg> 
  {
    // send keycode to dialog if there is a dialog.
    if let Some((m, d)) = &mut self.dlg {
      // process response
      match d.update(kc) {

        Some(InputMsg::Yes) => {
          let msg = Some(m.clone());
          self.dlg = None;
          msg
        }

        Some(InputMsg::No) => {
          self.dlg = None;
          Some(ViewMsg::Default)
        }

        Some(InputMsg::Ack) => {
          let msg = Some(m.clone());
          self.dlg = None;
          msg
        }

        Some(InputMsg::Text(text)) => {
          let msg = 
            if let ViewMsg::NewTab = m {
              Some(ViewMsg::Go(text))
            } else {
              Some(m.clone())
            };
          self.dlg = None;
          msg
        }

        Some(InputMsg::Cancel) => {
          self.dlg = None;
          Some(ViewMsg::Default)
        }

        Some(_) => {
          Some(ViewMsg::Default)
        }

        _ => 
          None
      }
    // there is no dialog, process keycode
    } else if kc == &usr.keys.global {
        Some(ViewMsg::Global)

    } else if kc == &usr.keys.move_down {
      self.ddoc
        .move_down(&self.frame, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.move_up {
      self.ddoc
        .move_up(&self.frame, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.move_left {
      self.ddoc
        .move_left(&self.frame, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.move_right {
      self.ddoc
        .move_right(&self.frame, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.cycle_left {
      Some(ViewMsg::CycleLeft)

    } else if kc == &usr.keys.cycle_right {
      Some(ViewMsg::CycleRight)

    // make a dialog
    } else if kc == &usr.keys.delete_tab {

      let dlg = Dialog::ask(
        &self.frame, 
        usr, 
        "Delete current tab?");
      self.dlg = Some((ViewMsg::DeleteMe, dlg));
      Some(ViewMsg::Default)

    } else if kc == &usr.keys.new_tab {

      let dlg = Dialog::text(
        &self.frame, 
        usr, 
        "enter path: ");
      self.dlg = Some((ViewMsg::NewTab, dlg));
      Some(ViewMsg::Default)

    } else if kc == &usr.keys.inspect {

      let gemtype = 
        match &self.gdoc {
          Some(gdoc) => {
            let idx = self.ddoc
              .select(&self.frame)
              .unwrap_or(0);

            gdoc.doc[idx].tag.clone()
          }
          None => 
            GemTag::Text,
        };

      let dialog_tuple = 
        match gemtype {
          GemTag::Link(Scheme::Gemini, url) => {
            let dlg = Dialog::ask(
              &self.frame, 
              usr, 
              &format!("go to {}?", url));
            (ViewMsg::Go(url.into()), dlg)
          }

          GemTag::Link(_, url) => {
            let dlg = Dialog::ack(
              &self.frame, 
              usr, 
              &format!(
                "Protocol {} not yet supported", 
                url));
            (ViewMsg::Default, dlg)
          }

          gemtext => {
            let dlg = Dialog::ack(
              &self.frame, 
              usr, 
              &format!(
                "you've selected {:?}", 
                gemtext));

            (ViewMsg::Default, dlg)
          }
        };

        self.dlg = Some(dialog_tuple);
        Some(ViewMsg::Default)

      } else {
        None
      }
  }

  pub fn update_usr(&mut self, _usr: &User) {
  }

  // show dialog if there's a dialog, otherwise show ddoc
  pub fn view(&self, writer: &mut impl Write) 
    -> io::Result<()> 
  {
    if let Some((_, d)) = &self.dlg {
      d.view(writer)?;

    } else {
      self.ddoc.view(&self.frame, writer)?;
    }
    Ok(())
  }

  // might display dialog
  fn some_gem_doc(&mut self, usr: &User, gemdoc: GemDoc) 
  {
    self.dlg = match gemdoc.status.tag {

      Status::InputExpected |
      Status::InputExpectedSensitive => {

        let dlg = 
          Dialog::text(&self.frame, usr, &gemdoc.status.txt);
        Some((ViewMsg::Reply, dlg))
      }

      Status::RedirectTemporary |
      Status::RedirectPermanent => {

        let dlg = 
          Dialog::ask(&self.frame, usr, &gemdoc.status.txt);
        Some((ViewMsg::Go(gemdoc.status.txt.clone()), dlg))
      }

      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => {

        let dlg = 
          Dialog::ack(&self.frame, usr, &gemdoc.status.txt);
        Some((ViewMsg::Default, dlg))
      }

      _ => {
        None
      }
    };

    self.ddoc = usr.get_doc(&gemdoc, &self.frame.outer);

    self.gdoc = Some(gemdoc);
  }

  // display dialog
  fn none_gem_doc(&mut self, usr: &User, msg: &str) {

    self.gdoc = None;

    let dlg  = Dialog::ack(&self.frame, usr, msg);

    self.dlg = Some((ViewMsg::DeleteMe, dlg));
  }

  fn make_request(&mut self, usr: &User, url_str: &str) 
  {
    match Url::parse(url_str) {

      Ok(url) => match GemDoc::new(&url) {

        Ok(gemdoc) => 
          self.some_gem_doc(usr, gemdoc),

        Err(e) => 
          self.none_gem_doc(usr, &e),
      }

      Err(e) => 
        self.none_gem_doc(usr, &e.to_string()),
    }
  }

}
