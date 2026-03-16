// src/tab.rs

use crate::{
  usr::{User},
  util::{Scheme},
  gem::{GemDoc, GemTag, Status},
  text::{Doc},
  page::{Page},
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
  pub page:  Page,
  pub name:  String,
  pub dlg:   Option<(ViewMsg, Dialog)>,
  pub gdoc:  Option<GemDoc>,
  pub ddoc:  Doc,
} 
impl Tab {

  pub fn init(page: &Page, url_str: &str, usr: &User) 
    -> Self 
  {
    let mut tab = Self {
      dlg:    None,
      gdoc:   None,
      ddoc:   Doc::default(), 
      page:   page.clone(),
      name:   url_str.into(),
    };
    tab.make_request(usr, url_str);
    tab
  }


  // resize ddoc and dialog
  pub fn resize(&mut self, page: &Page) {

    self.page = page.clone();
    self.ddoc.resize(page);

    if let Some((_, d)) = &mut self.dlg {
      d.resize(page);
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
        .move_down(&self.page, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.move_up {
      self.ddoc
        .move_up(&self.page, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.move_left {
      self.ddoc
        .move_left(&self.page, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.move_right {
      self.ddoc
        .move_right(&self.page, 1)
        .then_some(ViewMsg::Default)

    } else if kc == &usr.keys.cycle_left {
      Some(ViewMsg::CycleLeft)

    } else if kc == &usr.keys.cycle_right {
      Some(ViewMsg::CycleRight)

    // make a dialog
    } else if kc == &usr.keys.delete_tab {

      let dlg = usr.ask(
        &self.page, 
        "Delete current tab?");
      self.dlg = Some((ViewMsg::DeleteMe, dlg));
      Some(ViewMsg::Default)

    } else if kc == &usr.keys.new_tab {

      let dlg = usr.text(
        &self.page, 
        "enter path: ");
      self.dlg = Some((ViewMsg::NewTab, dlg));
      Some(ViewMsg::Default)

    } else if kc == &usr.keys.inspect {

      let gemtype = 
        match &self.gdoc {
          Some(gdoc) => {
            let idx = self.ddoc
              .select(&self.page)
              .unwrap_or(0);

            gdoc.doc[idx].tag.clone()
          }
          None => 
            GemTag::Text,
        };

      let dialog_tuple = 
        match gemtype {
          GemTag::Link(Scheme::Gemini, url) => {
            let dlg = usr.ask(
              &self.page, 
              &format!("go to {}?", url));
            (ViewMsg::Go(url.into()), dlg)
          }

          GemTag::Link(_, url) => {
            let dlg = usr.ack(
              &self.page, 
              &format!(
                "Protocol {} not yet supported", 
                url));
            (ViewMsg::Default, dlg)
          }

          gemtext => {
            let dlg = usr.ack(
              &self.page, 
              &format!("you've selected {:?}", gemtext));

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
      self.ddoc.view(&self.page, writer)?;
    }
    Ok(())
  }


  // might display dialog
  fn some_gem_doc(&mut self, usr: &User, gemdoc: GemDoc) 
  {
    self.dlg = match gemdoc.status.tag {

      Status::InputExpected |
      Status::InputExpectedSensitive => {

        let dlg = usr.text(&self.page, &gemdoc.status.txt);
        Some((ViewMsg::Reply, dlg))
      }

      Status::RedirectTemporary => {

        let dlg = usr.ask(&self.page, &gemdoc.status.txt);

        let new_url = gemdoc.url
          .join(&gemdoc.status.txt)
          .unwrap_or(gemdoc.url.clone());

        Some((ViewMsg::Go(new_url.into()), dlg))
      }

      Status::RedirectPermanent => {
        let dlg = usr.ask(&self.page, &gemdoc.status.txt);
        Some((ViewMsg::Go(gemdoc.status.txt.clone()), dlg))
      }

      Status::CertRequiredClient |
      Status::CertRequiredTransient |
      Status::CertRequiredAuthorized => {

        let dlg = usr.ack(&self.page, &gemdoc.status.txt);
        Some((ViewMsg::Default, dlg))
      }

      _ => {
        None
      }
    };

    self.ddoc = usr.get_doc(&gemdoc, &self.page);

    self.gdoc = Some(gemdoc);
  }


  // display dialog
  fn none_gem_doc(&mut self, usr: &User, msg: &str) {

    self.gdoc = None;

    let dlg  = usr.ack(&self.page, msg);

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
