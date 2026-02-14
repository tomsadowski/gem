// src/app.rs

use crate::{
    cfg::{self, Config},
    screen::{Screen, Rect},
    msg::{Focus, ViewMsg},
    pos::{Pos},
    reader::{DisplayDoc, DisplayText},
    tab::Tab,
};
use crossterm::{
    QueueableCommand, cursor,
    terminal::{Clear, ClearType},
    style::{Color},
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers}
};
use std::{
    io::{self, Write}
};

pub struct App {
    pub bg:         Color,
    pub hdr:        DisplayDoc,
    pub hdr_scr:    Screen,
    pub tabs:       Vec<Tab>,
    pub tab_scr:    Screen,
    pub idx:        usize,
    pub cfg_path:   String,
    pub cfg:        Config,
    pub focus:      Focus,
    pub quit:       bool,
} 
impl App {
    pub fn new(path: &str, w: u16, h: u16) -> Self {
        let cfg = cfg::load_config(path);
        let (hdr_scr, tab_scr) = Self::get_screens(w, h, &cfg);
        let tabs = vec![Tab::init(&tab_scr, &cfg.init_url, &cfg)];

        let mut app = Self {
            bg: cfg.colors.get_background(),
            cfg_path: path.into(),
            quit: false, 
            focus: Focus::Tab,
            idx: 0,
            hdr: DisplayDoc::default(&hdr_scr),
            hdr_scr, tab_scr,
            tabs,
            cfg,
        };

        app.update_hdr_text();
        app
    }

    pub fn view(&mut self, writer: &mut impl Write) -> io::Result<()> { 
        writer
            .queue(Clear(ClearType::All))?
            .queue(cursor::Hide)?;

        self.hdr.view(writer)?;
        self.tabs[self.idx].view(writer)?;

        writer
            .queue(cursor::Show)?
            .flush()
    }

    pub fn update(&mut self, event: Event) -> bool {
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
                        self.tabs[self.idx].update(&kc, &self.cfg),
                }; 
                if let Some(msg) = response { 
                    self.process_view_msg(msg);
                    self.update_hdr_text();
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn get_screens(w: u16, h: u16, cfg: &Config) -> (Screen, Screen) {
        let x_margin = cfg.format.margin;
        let y_margin = cfg.format.margin;
        let x_scroll = cfg.scroll_at;
        let y_scroll = cfg.scroll_at;
        let rect = Rect::new(w, h).crop_x(x_margin).crop_y(y_margin);
        let tab_rect = rect.crop_north(2);
        let hdr_rect = rect.crop_south(h - 2);
        let tab_scr = Screen::new(&tab_rect, x_scroll, y_scroll);
        let hdr_scr = Screen::new(&hdr_rect, 0, 0);
        (hdr_scr, tab_scr)
    }

    fn process_view_msg(&mut self, msg: ViewMsg) {
        match msg {
            ViewMsg::Global => {
                self.focus = Focus::Global;
            }
            ViewMsg::ReloadConfig => {
                self.update_cfg(cfg::load_config(&self.cfg_path));
            }
            ViewMsg::NewConfig(s) => {
                self.cfg_path = s;
                self.update_cfg(cfg::load_config(&self.cfg_path));
            }
            ViewMsg::Go(url) => {
                let tab = Tab::init(&self.tab_scr, &url, &self.cfg);
                self.tabs.push(tab);
                self.idx = self.tabs.len() - 1;
            }
            ViewMsg::DeleteMe => {
                if self.tabs.len() > 1 {
                    self.tabs.remove(self.idx);
                    self.idx = self.tabs.len() - 1;
                }
            }
            ViewMsg::CycleLeft => {
                if self.idx == 0 {
                    self.idx = self.tabs.len() - 1;
                } else {
                    self.idx -= 1;
                }
            }
            ViewMsg::CycleRight => {
                if self.idx == self.tabs.len() - 1 {
                    self.idx = 0;
                } else {
                    self.idx += 1;
                }
            }
            _ => {}
        }
    }

    fn resize(&mut self, w: u16, h: u16) {
        let (hdr_scr, tab_scr) = Self::get_screens(w, h, &self.cfg);
        self.hdr_scr = hdr_scr;
        self.tab_scr = tab_scr;
        for t in self.tabs.iter_mut() {
            t.resize(&self.tab_scr);
        }
        self.update_hdr_text();
    }

    fn update_hdr_text(&mut self) {
        let len = self.tabs.len();
        let idx = self.idx;
        let url = &self.tabs[idx].url;
        let text = &format!("{}/{}: {}", idx + 1, len, url);
        let width = self.hdr_scr.outer.w;
        let color = self.cfg.colors.get_banner();
        let vec = vec![
            DisplayText::new(&text, color, false),
            DisplayText::new(
                &String::from("-").repeat(width), color, false)];
        self.hdr = DisplayDoc::new(vec, &self.hdr_scr);
        self.hdr.update_view(&Pos::origin(&self.hdr_scr.outer)).unwrap();
    }

    fn update_cfg(&mut self, cfg: Config) {
        self.cfg = cfg;
        self.bg = self.cfg.colors.get_background();
        for t in self.tabs.iter_mut() {
            t.update_cfg(&self.cfg);
        }
    }

    fn update_global(&mut self, keycode: &KeyCode) -> Option<ViewMsg> {
        match keycode {
            KeyCode::Esc => {
                self.focus = Focus::Tab;
                Some(ViewMsg::Default)
            }
            KeyCode::Char(c) => {
                if c == &self.cfg.keys.tab_view {
                    self.focus = Focus::Tab;
                    Some(ViewMsg::Default)
                } else if c == &self.cfg.keys.load_cfg {
                    self.focus = Focus::Tab;
                    Some(ViewMsg::ReloadConfig)
                } else {None}
            } 
            _ => {None}
        }
    }
}
