// ui

use crate::{
    config::{self, Config, Colors},
    gemini::{GemType, GemDoc, Scheme},
    widget::{Rect, Pager, CursorText, ColoredText},
};
use crossterm::{
    QueueableCommand,
    terminal::{Clear, ClearType},
    cursor::{self, MoveTo},
    style::{self, Color, SetForegroundColor, SetBackgroundColor, Print},
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use std::{
    io::{self, Stdout, Write},
    fs,
};
use url::Url;

#[derive(Clone, Debug)]
pub enum ViewMsg {
    None,
    ReloadConfig,
    NewConfig(String),
}
#[derive(Clone, Debug)]
pub enum TabMsg {
    Quit,
    None,
    CycleLeft,
    CycleRight,
    DeleteMe,
    Acknowledge,
    NewTab,
    Open(String),
    Go(String),
}
#[derive(Clone, Debug)]
pub enum InputMsg {
    None,
    Cancel,
    Choose(char),
    Text(String),
}
// view currently in use
#[derive(Debug)]
pub enum View {
    Tab,
    Quit,
}
// coordinates activities between views
pub struct UI {
    rect:     Rect,
    cfg:      Config,
    cfg_path: String,
    bg_color: Color,
    view:     View,
    tabs:     TabServer,
} 
impl UI {
    // return default config if error
    fn load_config(path: &str) -> Config {
        match fs::read_to_string(path) {
            Ok(text) => Config::parse_or_default(&text),
            _        => Config::default(),
        }
    }
    // start with View::Tab
    pub fn new(path: &str, w: u16, h: u16) -> Self {
        let rect = Rect {x: 0, y: 0, w: w, h: h};
        let cfg = Self::load_config(path);
        Self {
            tabs:     TabServer::new(&rect, &cfg),
            rect:     rect,
            cfg_path: path.into(),
            cfg:      cfg.clone(),
            view:     View::Tab,
            bg_color: cfg.colors.get_background(),
        }
    }
    // resize all views, maybe do this in parallel?
    fn resize(&mut self, w: u16, h: u16) {
        self.rect = Rect {x: 0, y: 0, w: w, h: h};
        self.tabs.resize(&self.rect);
    }
    // display the current view
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(Clear(ClearType::All))?
            .queue(SetBackgroundColor(self.bg_color))?;
        match &self.view {
            View::Tab => self.tabs.view(stdout),
            _         => Ok(()),
        }?;
        stdout.flush()
    }
    // Resize and Control-C is handled here, 
    // otherwise delegate to current view
    pub fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Resize(w, h) => {
                self.resize(w, h); 
                true
            }
            Event::Key(
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code:      KeyCode::Char('c'),
                    kind:      KeyEventKind::Press, ..
                }
            ) => {
                self.view = View::Quit;
                true
            }
            Event::Key(
                KeyEvent {
                    code: keycode, 
                    kind: KeyEventKind::Press, ..
                }
            ) => {
                let view_msg = 
                    match &self.view {
                        View::Tab => 
                            self.tabs.update(&keycode),
                        _ => 
                            None,
                    }; 
                match view_msg {
                    Some(ViewMsg::ReloadConfig) => {
                        self.update_cfg(Self::load_config(&self.cfg_path));
                        true
                    }
                    Some(ViewMsg::NewConfig(s)) => {
                        self.cfg_path = s;
                        self.update_cfg(Self::load_config(&self.cfg_path));
                        true
                    }
                    Some(_) => 
                        true,
                    None => 
                        false
                } 
            }
            _ => false,
        }
    }
    fn update_cfg(&mut self, cfg: Config) {
        self.cfg = cfg;
        self.bg_color = self.cfg.colors.get_background();
        self.tabs.update_cfg(&self.cfg);
    }
    // no need to derive PartialEq for View
    pub fn is_quit(&self) -> bool {
        match self.view {View::Quit => true, _ => false}
    }
} 
pub struct TabServer {
    rect:     Rect,
    cfg:      Config,
    hdr_text: ColoredText,
    hdr_line: ColoredText,
    tabs:     Vec<Tab>,
    idx:      usize,
}
impl TabServer {
    pub fn new(rect: &Rect, cfg: &Config) -> Self {
        let rect = Self::get_rect(rect, cfg);
        Self {
            rect:     rect.clone(),
            cfg:      cfg.clone(),
            tabs:     vec![Tab::new(&rect, &cfg.init_url, cfg)],
            idx:      0,
            hdr_text: Self::get_hdr_text(0, 1, &cfg.init_url),
            hdr_line: Self::get_hdr_line(rect.w),
        }
    }
    // adjust length of banner line, resize all tabs
    pub fn resize(&mut self, rect: &Rect) {
        self.rect     = Self::get_rect(rect, &self.cfg);
        self.hdr_line = Self::get_hdr_line(self.rect.w);
        for tab in self.tabs.iter_mut() {
            tab.resize(&self.rect);
        }
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode) -> Option<ViewMsg> {
        let response = self.tabs[self.idx].update(keycode);
        if let Some(msg) = response {
            match msg {
                TabMsg::Go(url) => {
                    self.tabs.push(Tab::new(&self.rect, &url, &self.cfg));
                    self.idx = self.tabs.len() - 1;
                }
                TabMsg::Open(text) => {
                    self.tabs.push(Tab::new(&self.rect, &text, &self.cfg));
                    self.idx = self.tabs.len() - 1;
                }
                TabMsg::DeleteMe => {
                    if self.tabs.len() > 1 {
                        self.tabs.remove(self.idx);
                        self.idx = self.tabs.len() - 1;
                    }
                }
                TabMsg::CycleLeft => {
                    match self.idx == 0 {
                        true  => self.idx = self.tabs.len() - 1,
                        false => self.idx -= 1,
                    }
                }
                TabMsg::CycleRight => {
                    match self.idx == self.tabs.len() - 1 {
                        true  => self.idx = 0,
                        false => self.idx += 1,
                    }
                }
                _ => {},
            }
            let len = self.tabs.len();
            let url = self.tabs[self.idx].get_url();
            self.hdr_text = Self::get_hdr_text(self.idx, len, url);
            self.hdr_line = Self::get_hdr_line(self.rect.w);
            Some(ViewMsg::None)
        } else {
            None
        }
    }
    // display banner and page
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(MoveTo(self.rect.x, 0))?
            .queue(SetForegroundColor(self.hdr_text.color))?
            .queue(Print(&self.hdr_text.text))?
            .queue(MoveTo(self.rect.x, 1))?
            .queue(SetForegroundColor(self.hdr_line.color))?
            .queue(Print(&self.hdr_line.text))?;
        self.tabs[self.idx].view(stdout)
    }
    pub fn update_cfg(&mut self, cfg: &Config) {
        self.cfg      = cfg.clone();
        self.rect     = Self::get_rect(&self.rect, &self.cfg);
        self.hdr_line = Self::get_hdr_line(self.rect.w);
        for tab in self.tabs.iter_mut() {
            tab.update_cfg(&self.cfg);
        }
    }
    fn get_rect(rect: &Rect, cfg: &Config) -> Rect {
        Rect {
            x: rect.x + cfg.format.margin as u16, 
            y: rect.y + 2, 
            w: rect.w - (cfg.format.margin * 2) as u16,
            h: rect.h - 2
        }
    }
    fn get_hdr_text(idx: usize, total_tab: usize, url: &str) -> ColoredText {
        ColoredText::white(&format!("{}/{}: {}", idx + 1, total_tab, url))
    }
    fn get_hdr_line(w: u16) -> ColoredText {
        ColoredText::white(&String::from("-").repeat(usize::from(w)))
    }
}
pub struct Tab {
    rect: Rect,
    cfg:  Config,
    url:  String,
    doc:  Option<GemDoc>,
    dlg:  Option<(TabMsg, Dialog)>,
    page: Pager,
}
impl Tab {
    pub fn new(rect: &Rect, url_str: &str, cfg: &Config) -> Self {
        let doc = match Url::parse(url_str) {
            Ok(url) => GemDoc::new(&url).ok(),
            _       => None,
        };
        let page = Self::get_page(rect, &doc, cfg);
        Self {
            url:  String::from(url_str),
            cfg:  cfg.clone(),
            rect: rect.clone(),
            dlg:  None,
            page: page,
            doc:  doc,
        }
    }
    // resize page and all dialogs
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.page.resize(&rect);
        if let Some((_, d)) = &mut self.dlg {
            d.resize(&rect);
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<TabMsg> {
        // send keycode to dialog if there is a dialog
        if let Some((m, d)) = &mut self.dlg {
            match d.update(keycode) {
                Some(InputMsg::Choose(c)) => {
                    let msg = match c == self.cfg.keys.yes {
                        true  => Some(m.clone()),
                        false => Some(TabMsg::None),
                    };
                    self.dlg = None;
                    return msg
                }
                Some(InputMsg::Text(text)) => {
                    let msg = match m {
                        TabMsg::NewTab => 
                            Some(TabMsg::Open(text)),
                        _ => 
                            Some(TabMsg::None),
                    };
                    self.dlg = None;
                    return msg
                }
                Some(InputMsg::Cancel) => {
                    self.dlg = None;
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
            if c == &self.cfg.keys.move_cursor_down {
                match self.page.move_down(1) {
                    true  => return Some(TabMsg::None),
                    false => return None,
                }
            }
            else if c == &self.cfg.keys.move_cursor_up {
                match self.page.move_up(1) {
                    true  => return Some(TabMsg::None),
                    false => return None,
                }
            }
            else if c == &self.cfg.keys.cycle_to_left_tab {
                return Some(TabMsg::CycleLeft)
            }
            else if c == &self.cfg.keys.cycle_to_right_tab {
                return Some(TabMsg::CycleRight)
            }
            // make a dialog
            else if c == &self.cfg.keys.delete_current_tab {
                let dialog = 
                    Dialog::choose(
                        &self.rect,
                        "Delete current tab?",
                        vec![(self.cfg.keys.yes, "yes"),
                             (self.cfg.keys.no, "no")]);
                self.dlg = Some((TabMsg::DeleteMe, dialog));
                return Some(TabMsg::None)
            }
            else if c == &self.cfg.keys.new_tab {
                let dialog = Dialog::text(&self.rect, "enter path: ");
                self.dlg = Some((TabMsg::NewTab, dialog));
                return Some(TabMsg::None)
            }
            else if c == &self.cfg.keys.inspect_under_cursor {
                let gemtype = match &self.doc {
                    Some(doc) => 
                        doc.doc[self.page.select_under_cursor().0].0.clone(),
                    None => GemType::Text,
                };
                let dialog_tuple = 
                    match gemtype {
                        GemType::Link(Scheme::Gemini, url) => {
                            let dialog = Dialog::choose(
                                &self.rect,
                                &format!("go to {}?", url),
                                vec![(self.cfg.keys.yes, "yes"), 
                                     (self.cfg.keys.no, "no")]);
                            (TabMsg::Go(url.to_string()), dialog)
                        }
                        GemType::Link(_, url) => {
                            let dialog = Dialog::choose(
                                &self.rect,
                                &format!("Protocol {} not yet supported", url),
                                vec![(self.cfg.keys.yes, "acknowledge")]);
                            (TabMsg::Acknowledge, dialog)
                        }
                        gemtext => {
                            let dialog = Dialog::choose(
                                &self.rect,
                                &format!("you've selected {:?}", gemtext),
                                vec![(self.cfg.keys.yes, "acknowledge")]);
                            (TabMsg::Acknowledge, dialog)
                        }
                    };
                self.dlg = Some(dialog_tuple);
                return Some(TabMsg::None)
            } else {
                return None
            }
        } else {
            return None
        }
    }
    pub fn update_cfg(&mut self, cfg: &Config) {
        self.cfg = cfg.clone();
        self.page = Self::get_page(&self.rect, &self.doc, &self.cfg);
    }
    pub fn get_url(&self) -> &str {
        &self.url
    }
    // show dialog if there's a dialog, otherwise show page
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        match &self.dlg {
            Some((_, d)) => 
                d.view(stdout),
            _ => 
                self.page.view(stdout),
        }
    }
    fn get_page(rect: &Rect, doc: &Option<GemDoc>, cfg: &Config) -> Pager {
        let colored_text = 
            match &doc {
                Some(gemdoc) => 
                    cfg.colors.from_gem_doc(&gemdoc),
                None => 
                    vec![
                        cfg.colors.from_gem_type(
                            &GemType::Text, 
                            "Nothing to display")]
            };
        Pager::new(rect, &colored_text, cfg.scroll_at)
    }
}
#[derive(Clone, Debug)]
pub enum InputType {
    Choose {keys: Vec<char>, view: Pager},
    Text(CursorText),
}
impl InputType {
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match self {
            InputType::Text(cursortext) => {
                match keycode {
                    KeyCode::Enter => {
                        Some(InputMsg::Text(cursortext.get_text()))
                    }
                    KeyCode::Left => {
                        match cursortext.move_left(1) {
                            true  => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Right => {
                        match cursortext.move_right(1) {
                            true  => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Delete => {
                        match cursortext.delete() {
                            true  => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Backspace => {
                        match cursortext.backspace() {
                            true  => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Char(c) => {
                        cursortext.insert(*c);
                        Some(InputMsg::None)
                    }
                    _ => { 
                        None
                    }
                }
            }
            InputType::Choose {keys, ..} => {
                match keycode {
                    KeyCode::Char(c) => {
                        match keys.contains(&c) {
                            true  => Some(InputMsg::Choose(*c)),
                            false => None,
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct Dialog {
    rect:       Rect,
    prompt:     String,
    input_type: InputType,
}
impl Dialog {
    pub fn text(rect: &Rect, prompt: &str) -> Self {
        Self {
            rect:       rect.clone(),
            prompt:     String::from(prompt), 
            input_type: InputType::Text(CursorText::new(rect, "")),
        }
    }
    pub fn choose(  rect:   &Rect, 
                    prompt: &str, 
                    choose: Vec<(char, &str)>) -> Self
    {
        let view_rect = Rect {  x: rect.x, 
                                y: rect.y + 8, 
                                w: rect.w, 
                                h: rect.h - 8   };
        let keys_vec = choose.iter().map(|(c, _)| *c).collect();
        let view_vec = choose.iter()
                .map(|(x, y)| format!("|{}|  {}", x, y)).collect();
        let pager    = Pager::white(&view_rect, &view_vec);
        Self {
            rect:       rect.clone(),
            prompt:     String::from(prompt), 
            input_type: InputType::Choose { keys: keys_vec, 
                                            view: pager   },
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(MoveTo(self.rect.x, self.rect.y + 4))?
            .queue(Print(&self.prompt))?;
        match &self.input_type {
            InputType::Choose {view, ..} => {
                view.view(stdout)
            }
            InputType::Text(cursortext) => {
                cursortext.view(self.rect.y + 8, stdout)
            }
        }
    }
    // No wrapping yet, so resize is straightforward
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        match &mut self.input_type {
            InputType::Choose {view, ..} => {
                view.resize(&self.rect)
            }
            InputType::Text(cursortext) => {
                cursortext.resize(&self.rect)
            }
        }
    }
    // Keycode has various meanings depending on the InputType.
    // The match statement might be moved to impl InputType
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => 
                Some(InputMsg::Cancel),
            _ => 
                self.input_type.update(keycode)
        }
    }
}
