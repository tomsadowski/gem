// tabserver

use crate::{
    gemini::{GemDoc},
    widget::{ColoredText},
    config::{Config},
    geometry::{Rect},
    tab::{TabMsg, Tab},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    event::{KeyCode},
    style::{self},
};
use std::{
    io::{self, Stdout},
};
use url::Url;

pub struct TabServer {
    rect: Rect,
    config: Config,
    tabs: Vec<Tab>,
    // index of current tab
    curindex: usize,
    // header/banner
    bannertext: ColoredText,
    bannerline: ColoredText,
}
impl TabServer {
    pub fn new(rect: &Rect, config: &Config) -> Self {
        let rect = Rect::new(rect.x, rect.y + 2, rect.w, rect.h - 1);
        // TODO produce dialog if failed url
        let url = Url::parse(&config.init_url).unwrap();
        let doc = GemDoc::new(&url);
        Self {
            rect: rect.clone(),
            config: config.clone(),
            tabs: vec![Tab::new(&rect, doc, config)],
            curindex: 0,
            bannertext: Self::bannertext(0, 1, &url),
            bannerline: Self::bannerline(rect.w),
        }
    }
    // adjust length of banner line, resize all tabs
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = Rect::new(rect.x, rect.y + 2, rect.w, rect.h - 1);
        self.bannerline = Self::bannerline(rect.w);
        for d in self.tabs.iter_mut() {
            d.resize(&self.rect);
        }
    }
    // display banner and page
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?
            .queue(style::SetForegroundColor(self.bannertext.color))?
            .queue(style::Print(&self.bannertext.text))?
            .queue(cursor::MoveTo(0, 1))?
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
                        self.tabs.push(Tab::new(&self.rect, doc, &self.config));
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
                            true => self.curindex = self.tabs.len() - 1,
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
                self.bannertext = Self::bannertext(self.curindex, len, url);
                self.bannerline = Self::bannerline(self.rect.w);
                true
            }
            None => false,
        }
    }
    fn bannertext(curindex: usize, totaltab: usize, url: &Url) -> ColoredText {
        ColoredText::white(&format!("{}/{}: {}", curindex + 1, totaltab, url))
    }
    fn bannerline(w: u16) -> ColoredText {
        ColoredText::white(&String::from("-").repeat(usize::from(w)))
    }
}
