// gem/src/tabserver
use crate::{
    config::Config,
    widget::Rect,
    tab::{TabMsg, Tab},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    event::{KeyCode},
    style::{self, Colors, Color},
};
use std::{
    io::{self, Stdout},
};
use url::Url;

// Serves config info to new tabs and 
// oooooooOOOOOOOOOOOOOOO
pub struct TabServer {
    rect: Rect,
    config: Config,
    tabs: Vec<Tab>,
    // index of current tab
    curindex: usize,
    // meta data to display at all times
    bannerstr: String,
    bannerstrcolor: Colors,
    // separate banner from page
    bannerline: String,
    bannerlinecolor: Colors,
}
impl TabServer {
    pub fn new(rect: &Rect, config: &Config) -> Self {
        let rect = Rect::new(rect.x, rect.y + 2, rect.w, rect.h - 1);
        let url = Url::parse(&config.init_url).unwrap();
        Self {
            rect: rect.clone(),
            config: config.clone(),
            tabs: vec![Tab::new(&rect, &url, config)],
            curindex: 0,
            bannerstr: Self::bannerstr(0, 1, &url),
            bannerline: Self::bannerline(rect.w),
            bannerstrcolor: Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0}),
            bannerlinecolor: Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0}),
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
            .queue(style::SetColors(self.bannerstrcolor))?
            .queue(style::Print(self.bannerstr.as_str()))?
            .queue(cursor::MoveTo(0, 1))?
            .queue(style::SetColors(self.bannerlinecolor))?
            .queue(style::Print(&self.bannerline))?;
        self.tabs[self.curindex].view(stdout)
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode) -> bool {
        match self.tabs[self.curindex].update(keycode) {
            Some(msg) => {
                match msg {
                    TabMsg::Go(p) => {
                        let url = Url::parse(p.as_str()).unwrap();
                        self.tabs.push(Tab::new(&self.rect, &url, &self.config));
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
                let url = &self.tabs[self.curindex].url;
                self.bannerstr = Self::bannerstr(self.curindex, len, url);
                self.bannerline = Self::bannerline(self.rect.w);
                true
            }
            None => false,
        }
    }
    fn bannerstr(curindex: usize, totaltab: usize, url: &Url) -> String {
        format!("{}/{}: {}", curindex + 1, totaltab, url)
    }
    fn bannerline(w: u16) -> String {
        String::from("-").repeat(usize::from(w))
    }
}
