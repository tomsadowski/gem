// src/app.rs

use crate::{
    cfg::{self, Config},
    screen::{Screen, Rect},
    msg::{Focus, ViewMsg},
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
    pub scr:        Screen,
    pub hdr:        DisplayDoc,
    pub bg:         Color,
    pub tabs:       Vec<Tab>,
    pub idx:        usize,
    pub cfg_path:   String,
    pub cfg:        Config,
    pub focus:      Focus,
    pub quit:       bool,
} 
impl App {
    pub fn new(path: &str, w: u16, h: u16) -> Self {
        let cfg = cfg::load_config(path);
        let scr = Screen::new(&Rect::new(w, h).crop_north(8), 3, 3);
        let tabs = vec![Tab::init(&scr, &cfg.init_url, &cfg)];
        let mut app = Self {
            bg: cfg.colors.get_background(),
            cfg_path: path.into(),
            quit: false, 
            focus: Focus::Tab,
            idx: 0,
            hdr: DisplayDoc::default(&scr),
            scr,
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
                let tab_scr = 
                    Screen::new(&Rect::new(w, h).crop_north(8), 3, 3);
                for t in self.tabs.iter_mut() {
                    t.resize(&tab_scr);
                }
                true
            }
            Event::Key(
                KeyEvent {
                    code: kc, 
                    kind: KeyEventKind::Press, ..
                }
            ) => {
                let view_msg = match &self.focus {
                    Focus::Global => self.update_global(&kc),
                    Focus::Tab => self.tabs[self.idx].update(&kc, &self.cfg),
                }; 
                match view_msg {
                    Some(ViewMsg::Global) => {
                        self.focus = Focus::Global;
                        false
                    }
                    Some(ViewMsg::ReloadConfig) => {
                        self.update_cfg(cfg::load_config(&self.cfg_path));
                        true
                    }
                    Some(ViewMsg::NewConfig(s)) => {
                        self.cfg_path = s;
                        self.update_cfg(cfg::load_config(&self.cfg_path));
                        true
                    }
                    Some(_) => true,
                    None => false
                } 
            }
            _ => false,
        }
    }

    fn update_hdr_text(&mut self) {
        let len = self.tabs.len();
        let idx = self.idx;
        let url = &self.tabs[idx].url;
        let text = &format!("{}/{}: {}", idx + 1, len, url);
        let width = self.scr.outer.w;
        let color = self.cfg.colors.get_banner();
        let vec = vec![
            DisplayText::new(&text, color, false),
            DisplayText::new(
                &String::from("-").repeat(width), color, false)];
        self.hdr = DisplayDoc::new(vec, &self.scr);
    }

    fn update_cfg(&mut self, cfg: Config) {
        self.cfg = cfg;
        self.bg = self.cfg.colors.get_background();
//        self.tabs.update_cfg(&self.scr, &self.cfg);
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
