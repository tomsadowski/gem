// src/tab.rs

use crate::{
    cfg::{Config},
    util::{Scheme},
    gem::{GemDoc, GemType, Status},
    text::{Doc},
    screen::{Frame},
    pos::{Pos},
    msg::{ViewMsg, InputMsg},
    dlg::{Dialog},
};
use crossterm::{
    QueueableCommand, cursor,
    event::{KeyCode}
};
use std::{
    io::{self, Write}
};
use url::{Url};

pub struct Tab {
    pub pos:    Pos,
    pub frame:  Frame,
    pub url:    String,
    pub dlg:    Option<(ViewMsg, Dialog)>,
    pub doc:    Option<GemDoc>,
    pub ddoc:   Doc,
} 
impl Tab {
    // might display dialog
    fn some_gem_doc(&mut self, cfg: &Config, gemdoc: GemDoc) {
        self.dlg = match gemdoc.status {
            Status::InputExpected |
            Status::InputExpectedSensitive => {
                let dlg = 
                    Dialog::text(&self.frame, cfg, &gemdoc.msg);
                Some((ViewMsg::Reply, dlg))
            }
            Status::RedirectTemporary |
            Status::RedirectPermanent => {
                let dlg = 
                    Dialog::ask(&self.frame, cfg, &gemdoc.msg);
                Some((ViewMsg::NewTab, dlg))
            }
            Status::CertRequiredClient |
            Status::CertRequiredTransient |
            Status::CertRequiredAuthorized => {
                let dlg = 
                    Dialog::ack(&self.frame, cfg, &gemdoc.msg);
                Some((ViewMsg::Default, dlg))
            }
            _ => {None}
        };
        let text = cfg.colors.from_gem_doc(&gemdoc);
        self.ddoc = Doc::new(text, &self.frame);
        self.doc = Some(gemdoc);
    }
    // display dialog
    fn none_gem_doc(&mut self, cfg: &Config, msg: &str) {
        let dlg  = Dialog::ack(&self.frame, cfg, msg);
        self.doc = None;
        self.dlg = Some((ViewMsg::Default, dlg));
    }
    fn make_request(&mut self, cfg: &Config) {
        let url = Url::parse(&self.url);
        match url {
            Ok(url) => match GemDoc::new(&url) {
                Ok(gemdoc) => {
                    self.some_gem_doc(cfg, gemdoc);
                }
                Err(e) => {
                    self.none_gem_doc(cfg, &e)
                }
            }
            Err(e) => {
                self.none_gem_doc(cfg, &e.to_string())
            }
        }
    }
    pub fn init(frame: &Frame, url_str: &str, cfg: &Config) -> Self {
        let pos = Pos::origin(&frame.outer);
        let frame = frame.clone();
        let mut tab = Self {
            url:  url_str.into(),
            dlg:  None,
            ddoc: Doc::default(), 
            doc: None,
            pos, 
            frame: frame.clone(),
        };
        tab.make_request(cfg);
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

    pub fn update(&mut self, kc: &KeyCode, cfg: &Config) -> Option<ViewMsg> {
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
                    let msg = if let ViewMsg::NewTab = m {
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
               _ => None
            }
        }
        // there is no dialog, process keycode
        else if let KeyCode::Char(c) = kc {
            if c == &cfg.keys.global {
                Some(ViewMsg::Global)
            }
            else if c == &cfg.keys.tab.move_down {
                self.pos.move_down(&self.frame, &self.ddoc, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_up {
                self.pos.move_up(&self.frame, &self.ddoc, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_left {
                self.pos.move_left(&self.frame, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_right {
                self.pos.move_right(&self.frame, &self.ddoc, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.cycle_left {
                Some(ViewMsg::CycleLeft)
            }
            else if c == &cfg.keys.tab.cycle_right {
                Some(ViewMsg::CycleRight)
            }
            // make a dialog
            else if c == &cfg.keys.tab.delete_tab {
                let dialog = Dialog::ask(
                    &self.frame, cfg, "Delete current tab?");
                self.dlg = Some((ViewMsg::DeleteMe, dialog));
                Some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.new_tab {
                let dialog = Dialog::text(
                    &self.frame, cfg, "enter path: ");
                self.dlg = Some((ViewMsg::NewTab, dialog));
                Some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.inspect {
                let gemtype = match &self.doc {
                    Some(doc) => {
                        let idx = 
                            self.ddoc.select(&self.frame, &self.pos)
                                .unwrap_or(0);
                        doc.doc[idx].0.clone()
                    }
                    None => GemType::Text,
                };
                let dialog_tuple = match gemtype {
                    GemType::Link(Scheme::Gemini, url) => {
                        let msg = &format!("go to {}?", url);
                        let dialog = Dialog::ask(&self.frame, cfg, &msg);
                        (ViewMsg::Go(url.into()), dialog)
                    }
                    GemType::Link(_, url) => {
                        let msg = 
                            format!("Protocol {} not yet supported", url);
                        let dialog = Dialog::ack(&self.frame, cfg, &msg);
                        (ViewMsg::Default, dialog)
                    }
                    gemtext => {
                        let msg = format!("you've selected {:?}", gemtext);
                        let dialog = Dialog::ack(&self.frame, cfg, &msg);
                        (ViewMsg::Default, dialog)
                    }
                };
                self.dlg = Some(dialog_tuple);
                Some(ViewMsg::Default)
            } else {None}
        } else {None}
    }

    pub fn update_cfg(&mut self, cfg: &Config) {
        self.ddoc = Self::get_ddoc(&self.frame, &self.doc, cfg);
    }

    // show dialog if there's a dialog, otherwise show ddoc
    pub fn view(&self, writer: &mut impl Write) -> io::Result<()> {
        if let Some((_, d)) = &self.dlg {
            d.view(writer)?;
        } else {
            self.ddoc.get_page(&self.frame, Some(&self.pos)).view(writer)?;
            writer.queue(cursor::MoveTo
                (self.pos.x.cursor, self.pos.y.cursor))?;
        }
        Ok(())
    }

    fn get_ddoc(frame: &Frame, doc: &Option<GemDoc>, cfg: &Config) 
        -> Doc 
    {
        let txt = if let Some(gemdoc) = &doc {
            cfg.colors.from_gem_doc(&gemdoc) 
        } else {
            vec![
                cfg.colors.from_gem_type(
                    &GemType::Text, 
                    "Nothing to display")]
        };
        Doc::new(txt, frame)
    }
}
