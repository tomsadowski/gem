// src/tab.rs

use crate::{
    cfg::{Config},
    util::{Scheme},
    gem::{GemDoc, GemType},
    reader::{DisplayDoc},
    screen::{Screen},
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
    pub scr:    Screen,
    pub url:    String,
    pub dlg:    Option<(ViewMsg, Dialog)>,
    pub doc:    Option<GemDoc>,
    pub ddoc:   DisplayDoc,
} 
impl Tab {
    pub fn init(scr: &Screen, url_str: &str, cfg: &Config) -> Self {
        let doc = Url::parse(url_str).ok()
            .map(|url| GemDoc::new(&url).ok())
            .flatten();
        let ddoc = Self::get_ddoc(scr, &doc, cfg);
        let pos = Pos::origin(&scr.outer);
        let scr = scr.clone();
        let tab = Self {
            url:  String::from(url_str),
            dlg:  None,
            ddoc, doc, pos, scr,
        };
        tab
    }

    // resize ddoc and dialog
    pub fn resize(&mut self, scr: &Screen) {
        self.scr = scr.clone();
        self.ddoc.resize(scr);
        if let Some((_, d)) = &mut self.dlg {
            d.resize(scr);
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
                        Some(ViewMsg::Default)
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
                self.pos.move_down(&self.scr, &self.ddoc, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_up {
                self.pos.move_up(&self.scr, &self.ddoc, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_left {
                self.pos.move_left(&self.scr, 1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_right {
                self.pos.move_right(&self.scr, &self.ddoc, 1)
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
                    &self.scr, cfg, "Delete current tab?");
                self.dlg = Some((ViewMsg::DeleteMe, dialog));
                Some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.new_tab {
                let dialog = Dialog::text(
                    &self.scr, cfg, "enter path: ");
                self.dlg = Some((ViewMsg::NewTab, dialog));
                Some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.inspect {
                let gemtype = match &self.doc {
                    Some(doc) => {
                        let idx = self.ddoc.select(&self.pos).unwrap_or(0);
                        doc.doc[idx].0.clone()
                    }
                    None => GemType::Text,
                };
                let dialog_tuple = match gemtype {
                    GemType::Link(Scheme::Gemini, url) => {
                        let msg = &format!("go to {}?", url);
                        let dialog = Dialog::ask(&self.scr, cfg, &msg);
                        (ViewMsg::Go(url.into()), dialog)
                    }
                    GemType::Link(_, url) => {
                        let msg = 
                            format!("Protocol {} not yet supported", url);
                        let dialog = Dialog::ack(&self.scr, cfg, &msg);
                        (ViewMsg::Default, dialog)
                    }
                    gemtext => {
                        let msg = format!("you've selected {:?}", gemtext);
                        let dialog = Dialog::ack(&self.scr, cfg, &msg);
                        (ViewMsg::Default, dialog)
                    }
                };
                self.dlg = Some(dialog_tuple);
                Some(ViewMsg::Default)
            } else {None}
        } else {None}
    }

    pub fn update_cfg(&mut self, cfg: &Config) {
        self.ddoc = Self::get_ddoc(&self.scr, &self.doc, cfg);
    }

    // show dialog if there's a dialog, otherwise show ddoc
    pub fn view(&mut self, writer: &mut impl Write) -> io::Result<()> {
        if let Some((_, d)) = &mut self.dlg {
            d.view(writer)?;
        } else {
            self.ddoc.update_view(&self.pos)?;
            self.ddoc.view(writer)?;
        }
        writer.queue(cursor::MoveTo(self.pos.x.cursor, self.pos.y.cursor))?;
        Ok(())
    }

    fn get_ddoc(scr: &Screen, doc: &Option<GemDoc>, cfg: &Config) 
        -> DisplayDoc 
    {
        let txt = if let Some(gemdoc) = &doc {
            cfg.colors.from_gem_doc(&gemdoc) 
        } else {
            vec![
                cfg.colors.from_gem_type(
                    &GemType::Text, 
                    "Nothing to display")]
        };
        DisplayDoc::new(txt, scr)
    }
}
