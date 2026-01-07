// tab

use crate::{
    config::{self, Config},
    gemini::{GemType, GemDoc},
    util::{Rect},
    widget::{Selector, ColoredText},
    dialog::{Dialog, DialogMsg, InputType, InputMsg},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    event::{KeyCode},
    style::{self, Color},
};
use std::{
    io::{self, Stdout},
};
use url::Url;

pub struct TabServer {
    rect:       Rect,
    config:     Config,
    tabs:       Vec<Tab>,
    curindex:   usize,
    bgcolor:    Color,
    bannertext: ColoredText,
    bannerline: ColoredText,
}
impl TabServer {
    pub fn new(r: &Rect, config: &Config) -> Self {
        let rect = Rect {
            x: r.x + config.format.margin as u16, 
            y: r.y + 2, 
            w: r.w - (config.format.margin * 2) as u16,
            h: r.h - 2
        };
        let url = Url::parse(&config.init_url).unwrap();
        let doc = GemDoc::new(&url);
        Self {
            bgcolor:    config::getbackground(&config.colors),
            rect:       rect.clone(),
            config:     config.clone(),
            tabs:       vec![Tab::new(&rect, doc, config)],
            curindex:   0,
            bannertext: bannertext(0, 1, &url),
            bannerline: bannerline(rect.w),
        }
    }
    // adjust length of banner line, resize all tabs
    pub fn resize(&mut self, r: &Rect) {
        self.rect = Rect {
            x: r.x + self.config.format.margin as u16, 
            y: r.y + 2, 
            w: r.w - (self.config.format.margin * 2) as u16, 
            h: r.h - 2
        };
        self.bannerline = bannerline(self.rect.w);
        for d in self.tabs.iter_mut() {
            d.resize(&self.rect);
        }
    }
    // display banner and page
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x, 0))?
            .queue(style::SetBackgroundColor(self.bgcolor))?
            .queue(style::SetForegroundColor(self.bannertext.color))?
            .queue(style::Print(&self.bannertext.text))?
            .queue(cursor::MoveTo(self.rect.x, 1))?
            .queue(style::SetForegroundColor(self.bannerline.color))?
            .queue(style::Print(&self.bannerline.text))?;
        self.tabs[self.curindex].view(stdout)
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode) -> bool {
        match self.tabs[self.curindex].update(keycode) {
            Some(msg) => {
                match msg {
                    TabMsg::Go(url) => {
                        let doc = GemDoc::new(&url);
                        self.tabs.push(
                            Tab::new(&self.rect, doc, &self.config));
                        self.curindex = self.tabs.len() - 1;
                    }
                    TabMsg::DeleteMe => {
                        if self.tabs.len() > 1 {
                            self.tabs.remove(self.curindex);
                            self.curindex = self.tabs.len() - 1;
                        }
                    }
                    TabMsg::CycleLeft => {
                        match self.curindex == 0 {
                            true => 
                                self.curindex = self.tabs.len() - 1,
                            false => self.curindex -= 1,
                        }
                    }
                    TabMsg::CycleRight => {
                        match self.curindex == self.tabs.len() - 1 {
                            true => self.curindex = 0,
                            false => self.curindex += 1,
                        }
                    }
                    _ => {},
                }
                let len = self.tabs.len();
                let url = &self.tabs[self.curindex].doc.url;
                self.bannertext = 
                    bannertext(self.curindex, len, url);
                self.bannerline = bannerline(self.rect.w);
                true
            }
            None => false,
        }
    }
}
#[derive(Clone, Debug)]
pub enum TabMsg {
    Quit,
    None,
    CycleLeft,
    CycleRight,
    DeleteMe,
    Go(Url),
}
pub struct Tab {
    pub doc:  GemDoc,
    rect:     Rect,
    config:   Config,
    dlgstack: Vec<Dialog<TabMsg>>,
    page:     Selector,
}
impl Tab {
    pub fn new(rect: &Rect, gemdoc: GemDoc, config: &Config) 
        -> Self 
    {
        Self {
            config:   config.clone(),
            rect:     rect.clone(),
            dlgstack: vec![],
            page:     Selector::new(
                        rect, 
                        &config::getvec(&gemdoc.doc, &config.colors),
                        config.scroll_at),
            doc:      gemdoc,
        }
    }
    // resize page and all dialogs
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.page.resize(&rect);
        for d in self.dlgstack.iter_mut() {
            d.resize(&rect);
        }
    }
    // show dialog if there's a dialog, otherwise show page
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        match self.dlgstack.last() {
            Some(d) => d.view(stdout),
            _ => self.page.view(stdout),
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<TabMsg> {
        // send keycode to dialog if there is a dialog
        if let Some(d) = self.dlgstack.last_mut() {
            match d.update(keycode) {
                Some(DialogMsg::Submit(action, submission)) => {
                    let msg = match submission {
                        InputMsg::Choose(c) => {
                            match c == self.config.keys.yes {
                                true => Some(action),
                                false => Some(TabMsg::None),
                            }
                        }
                        InputMsg::Input(_) => Some(TabMsg::None),
                        _ => Some(TabMsg::None),
                    };
                    self.dlgstack.pop();
                    return msg
                }
                Some(DialogMsg::Cancel) => {
                    self.dlgstack.pop();
                    return Some(TabMsg::None)
                }
                Some(_) => {
                    return Some(TabMsg::None)
                }
               _ => return None
            }
        }
        // there is no dialog, process keycode here
        else if let KeyCode::Char(c) = keycode {
            if c == &self.config.keys.move_cursor_down {
                match self.page.move_down(1) {
                    true => return Some(TabMsg::None),
                    false => return None,
                }
            }
            else if c == &self.config.keys.move_cursor_up {
                match self.page.move_up(1) {
                    true => return Some(TabMsg::None),
                    false => return None,
                }
            }
            else if c == &self.config.keys.cycle_to_left_tab {
                return Some(TabMsg::CycleLeft)
            }
            else if c == &self.config.keys.cycle_to_right_tab {
                return Some(TabMsg::CycleRight)
            }
            // make a dialog
            else if c == &self.config.keys.delete_current_tab {
                let dialog = 
                    Dialog::new(
                        &self.rect,
                        TabMsg::DeleteMe,
                        InputType::choose(vec![
                            (self.config.keys.yes, "yes"),
                            (self.config.keys.no, "no")]),
                        "Delete current tab?");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.new_tab {
                let dialog = 
                    Dialog::new(
                        &self.rect,
                        TabMsg::None,
                        InputType::input(),
                        "enter path: ");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.inspect_under_cursor {
                let dialog = 
                    match &self.doc
                        .doc[self.page.select_under_cursor().0].0 
                    {
                        GemType::Link(_, url) => 
                            Dialog::new(
                                &self.rect,
                                TabMsg::Go(url.clone()),
                                InputType::choose(vec![
                                    (self.config.keys.yes, "yes"), 
                                    (self.config.keys.no, "no")]),
                                &format!("go to {}?", url)),
                        gemtext => 
                            Dialog::new(
                                &self.rect,
                                TabMsg::None,
                                InputType::None,
                                &format!("{:?}", gemtext)),
                    };
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            } else {
                return None
            }
        } else {
            return None
        }
    }
}
pub fn bannertext(curindex: usize, totaltab: usize, url: &Url) 
    -> ColoredText 
{
    ColoredText::white(
        &format!("{}/{}: {}", curindex + 1, totaltab, url))
}
pub fn bannerline(w: u16) -> ColoredText {
    ColoredText::white(&String::from("-").repeat(usize::from(w)))
}
