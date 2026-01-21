// ui

use crate::{
    config::{self, Config, ColorParams},
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
    Default,
    Global,
    ReloadConfig,
    Msg(String),
    NewConfig(String),
}
#[derive(Clone, Debug)]
pub enum TabMsg {
    Default,
    Quit,
    CycleLeft,
    CycleRight,
    DeleteMe,
    NewTab,
    Go(String),
    ViewMsg(ViewMsg)
}
#[derive(Clone, Debug)]
pub enum InputMsg {
    Default,
    Cancel,
    Ack,
    Yes,
    No,
    Text(String),
}
// view currently in use
#[derive(Debug, Clone)]
pub enum View {
    Tab,
    Msg,
    Quit,
}
// view currently in use
#[derive(Debug, Clone)]
pub enum Focus {
    View(View),
    Global,
}
// coordinates activities between views
pub struct UI {
    rect:     Rect,
    cfg:      Config,
    cfg_path: String,
    bg_color: Color,
    view:     View,
    focus:    Focus,
    msg:      MessageView,
    tabs:     TabView,
} 
impl UI {
    // return default config if error
    fn load_config(path: &str) -> (Config, ViewMsg) {
        match fs::read_to_string(path) {
            Ok(text) => 
                match Config::parse(&text) {
                    Ok(cfg) => {
                        (cfg, ViewMsg::Default)
                    }
                    Err(e) => {
                        (Config::default(), ViewMsg::Msg(e))
                    }
                }
            Err(e) => (Config::default(), ViewMsg::Msg(e.to_string())),
        }
    }
    // start with View::Tab
    pub fn new(path: &str, w: u16, h: u16) -> Self {
        let rect = Rect::origin(w, h);
        let mut view = View::Tab;
        let (cfg, cfgmsg) = Self::load_config(path);
        let mut msgview = MessageView::init(&rect, &cfg);
        if let ViewMsg::Msg(msg) = cfgmsg {
            msgview.push(&msg);
            view = View::Msg;
        };
        let (tabview, tabmsg) = TabView::init(&rect, &cfg);
        if let ViewMsg::Msg(msg) = tabmsg {
            msgview.push(&msg);
            view = View::Msg;
        };
        Self {
            focus:    Focus::View(view.clone()),
            tabs:     tabview,
            msg:      msgview,
            view:     view,
            rect:     rect,
            cfg_path: path.into(),
            cfg:      cfg.clone(),
            bg_color: cfg.colors.get_background(),
        }
    }
    // resize all views, maybe do this in parallel?
    fn resize(&mut self, w: u16, h: u16) {
        self.rect = Rect::origin(w, h);
        self.tabs.resize(&self.rect);
        self.msg.resize(&self.rect);
    }
    // display the current view
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(Clear(ClearType::All))?
            .queue(SetBackgroundColor(self.bg_color))?;
        match &self.view {
            View::Tab => self.tabs.view(stdout),
            View::Msg => self.msg.view(stdout),
            _         => Ok(()),
        }?;
        stdout.flush()
    }
    fn update_global(&mut self, keycode: &KeyCode) -> Option<ViewMsg> {
        match keycode {
            KeyCode::Esc => {
                self.focus = Focus::View(self.view.clone());
                return Some(ViewMsg::Default)
            }
            KeyCode::Char(c) => {
                if c == &self.cfg.keys.tab_view {
                    self.view = View::Tab;
                    self.focus = Focus::View(self.view.clone());
                    return Some(ViewMsg::Default)
                } else if c == &self.cfg.keys.msg_view {
                    self.view = View::Msg;
                    self.focus = Focus::View(self.view.clone());
                    return Some(ViewMsg::Default)
                } else if c == &self.cfg.keys.load_cfg {
                    self.focus = Focus::View(self.view.clone());
                    return Some(ViewMsg::ReloadConfig)
                } else {
                    return None
                }
            } 
            _ => {
                return None
            }
        }

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
                    match &self.focus {
                        Focus::View(view) => match view {
                            View::Tab => 
                                self.tabs.update(&keycode),
                            View::Msg => 
                                self.msg.update(&keycode),
                            _ => None,
                        },
                        Focus::Global => 
                            self.update_global(&keycode),
                    }; 
                match view_msg {
                    Some(ViewMsg::Global) => {
                        self.focus = Focus::Global;
                        false
                    }
                    Some(ViewMsg::ReloadConfig) => {
                        self.update_cfg(Self::load_config(&self.cfg_path).0);
                        true
                    }
                    Some(ViewMsg::NewConfig(s)) => {
                        self.cfg_path = s;
                        self.update_cfg(Self::load_config(&self.cfg_path).0);
                        true
                    }
                    Some(ViewMsg::Msg(s)) => {
                        self.msg.push(&s);
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
        self.tabs.update_cfg(&self.rect, &self.cfg);
    }
    // no need to derive PartialEq for View
    pub fn is_quit(&self) -> bool {
        match self.view {View::Quit => true, _ => false}
    }
} 
pub struct MessageView {
    rect:     Rect,
    cfg:      Config,
    messages: Vec<String>,
    page:     Pager,
}
impl MessageView {
    pub fn push(&mut self, msg: &str) {
        self.messages.push(msg.into());
        self.page = Pager::one_color( 
            &self.rect,
            &self.messages, 
            self.cfg.colors.get_dialog(),
            self.cfg.scroll_at);
    }
    pub fn init(rect: &Rect, cfg: &Config) -> Self {
        let page = Pager::one_color( 
            rect, 
            &vec![],
            cfg.colors.get_dialog(),
            cfg.scroll_at);
        Self {
            rect:       rect.clone(),
            cfg:        cfg.clone(),
            page:       page,
            messages:   vec![],
        }
    }
    // resize page
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.page.resize(&rect);
    }
    // show dialog if there's a dialog, otherwise show page
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        self.page.view(stdout)
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode) -> Option<ViewMsg> {
        if let KeyCode::Char(c) = keycode {
            if c == &self.cfg.keys.global {
                return Some(ViewMsg::Global)
            }
            if c == &self.cfg.keys.tab.move_down {
                match self.page.move_down(1) {
                    true  => return Some(ViewMsg::Default),
                    false => return None,
                }
            }
            else if c == &self.cfg.keys.tab.move_up {
                match self.page.move_up(1) {
                    true  => return Some(ViewMsg::Default),
                    false => return None,
                }
            } else {
                return None
            }
        } else {
            return None
        }
    }
}
pub struct TabView {
    rect:     Rect,
    cfg:      Config,
    hdr_text: ColoredText,
    hdr_line: ColoredText,
    tabs:     Vec<Tab>,
    idx:      usize,
}
impl TabView {
    pub fn init(rect: &Rect, cfg: &Config) -> (Self, ViewMsg) {
        let rect = Self::get_rect(rect, cfg);
        let (tab, msg) = Tab::init(&rect, &cfg.init_url, cfg);
        let tabview = Self {
            rect:     rect.clone(),
            cfg:      cfg.clone(),
            tabs:     vec![tab],
            idx:      0,
            hdr_text: Self::get_hdr_text(rect.w, &cfg, 0, 1, &cfg.init_url),
            hdr_line: Self::get_hdr_line(rect.w, &cfg),
        };
        (tabview, msg)
    }
    // adjust length of banner line, resize all tabs
    pub fn resize(&mut self, rect: &Rect) {
        self.rect     = Self::get_rect(rect, &self.cfg);
        self.hdr_line = Self::get_hdr_line(self.rect.w, &self.cfg);
        for tab in self.tabs.iter_mut() {
            tab.resize(&self.rect);
        }
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode) -> Option<ViewMsg> {
        let response = self.tabs[self.idx].update(keycode);
        if let Some(msg) = response {
            let mut viewmsg: Option<ViewMsg> = Some(ViewMsg::Default);
            match msg {
                TabMsg::ViewMsg(m) => {
                    viewmsg = Some(m);
                }
                TabMsg::Go(url) => {
                    let (tab, m) = Tab::init(&self.rect, &url, &self.cfg);
                    viewmsg = Some(m);
                    self.tabs.push(tab);
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
            self.hdr_text = 
                Self::get_hdr_text(
                    self.rect.w, 
                    &self.cfg, 
                    self.idx, 
                    len, 
                    &url);
            self.hdr_line = Self::get_hdr_line(self.rect.w, &self.cfg);
            viewmsg
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
    pub fn update_cfg(&mut self, rect: &Rect, cfg: &Config) {
        self.cfg      = cfg.clone();
        self.rect     = Self::get_rect(rect, &self.cfg);
        self.hdr_text = 
            Self::get_hdr_text(
                self.rect.w, 
                &self.cfg, 
                self.idx, 
                self.tabs.len(), 
                &self.tabs[self.idx].url);
        self.hdr_line = Self::get_hdr_line(self.rect.w, &self.cfg);
        for tab in self.tabs.iter_mut() {
            tab.update_cfg(&self.rect, &self.cfg);
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
    fn get_hdr_text(    w:          u16,
                        cfg:        &Config,
                        idx:        usize, 
                        total_tab:  usize, 
                        path:       &str    ) -> ColoredText 
    {
        let text = &format!("{}/{}: {}", idx + 1, total_tab, path);
        let width = std::cmp::min(usize::from(w), text.len());
        ColoredText::new(
                &text[..usize::from(width)],
                cfg.colors.get_banner()
            )
    }
    fn get_hdr_line(w: u16, cfg: &Config) -> ColoredText {
        ColoredText::new(
                &String::from("-").repeat(usize::from(w)),
                cfg.colors.get_banner()
            )
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
    pub fn init(rect: &Rect, url_str: &str, cfg: &Config) -> (Self, ViewMsg) {
        let mut msg = ViewMsg::Default;
        let mut doc: Option<GemDoc> = None;
        match Url::parse(url_str) {
            Err(e) => {
                msg = ViewMsg::Msg(e.to_string());
            }
            Ok(url) => {
                match GemDoc::new(&url) {
                    Ok(gemdoc) => {
                        doc = Some(gemdoc);
                    }
                    Err(e) => {
                        msg = ViewMsg::Msg(e.to_string());
                    }
                }
            }
        }
        let page = Self::get_page(rect, &doc, cfg);
        let tab = Self {
            url:  String::from(url_str),
            cfg:  cfg.clone(),
            rect: rect.clone(),
            dlg:  None,
            page: page,
            doc:  doc,
        };
        (tab, msg)
    }
    // resize page and dialog
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.page.resize(&rect);
        if let Some((_, d)) = &mut self.dlg {
            d.resize(&rect);
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<TabMsg> {
        // send keycode to dialog if there is a dialog.
        if let Some((m, d)) = &mut self.dlg {
            // process response
            match d.update(keycode) {
                Some(InputMsg::Yes) => {
                    let msg = Some(m.clone());
                    self.dlg = None;
                    return msg
                }
                Some(InputMsg::No) => {
                    self.dlg = None;
                    return Some(TabMsg::Default)
                }
                Some(InputMsg::Ack) => {
                    let msg = Some(m.clone());
                    self.dlg = None;
                    return msg
                }
                Some(InputMsg::Text(text)) => {
                    let msg = match m {
                        TabMsg::NewTab => Some(TabMsg::Go(text)),
                        _              => Some(TabMsg::Default),
                    };
                    self.dlg = None;
                    return msg
                }
                Some(InputMsg::Cancel) => {
                    self.dlg = None;
                    return Some(TabMsg::Default)
                }
                Some(_) => {
                    return Some(TabMsg::Default)
                }
               _ => return None
            }
        }
        // there is no dialog, process keycode
        else if let KeyCode::Char(c) = keycode {
            if c == &self.cfg.keys.global {
                return Some(TabMsg::ViewMsg(ViewMsg::Global))
            }
            if c == &self.cfg.keys.tab.move_down {
                match self.page.move_down(1) {
                    true  => return Some(TabMsg::Default),
                    false => return None,
                }
            }
            else if c == &self.cfg.keys.tab.move_up {
                match self.page.move_up(1) {
                    true  => return Some(TabMsg::Default),
                    false => return None,
                }
            }
            else if c == &self.cfg.keys.tab.cycle_left {
                return Some(TabMsg::CycleLeft)
            }
            else if c == &self.cfg.keys.tab.cycle_right {
                return Some(TabMsg::CycleRight)
            }
            // make a dialog
            else if c == &self.cfg.keys.tab.delete_tab {
                let dialog = Dialog::ask(
                    &self.rect,
                    &self.cfg,
                    "Delete current tab?");
                self.dlg = Some((TabMsg::DeleteMe, dialog));
                return Some(TabMsg::Default)
            }
            else if c == &self.cfg.keys.tab.new_tab {
                let dialog = Dialog::text(
                    &self.rect,
                    &self.cfg,
                    "enter path: ");
                self.dlg = Some((TabMsg::NewTab, dialog));
                return Some(TabMsg::Default)
            }
            else if c == &self.cfg.keys.tab.inspect {
                let gemtype = match &self.doc {
                    Some(doc) => 
                        doc.doc[self.page.select_under_cursor().0].0.clone(),
                    None => GemType::Text,
                };
                let dialog_tuple = 
                    match gemtype {
                        GemType::Link(Scheme::Gemini, url) => {
                            let dialog = Dialog::ask(
                                &self.rect, 
                                &self.cfg, 
                                &format!("go to {}?", url));
                            (TabMsg::Go(url.to_string()), dialog)
                        }
                        GemType::Link(_, url) => {
                            let dialog = Dialog::ack(
                                &self.rect,
                                &self.cfg,
                                &format!("Protocol {} not yet supported", url));
                            (TabMsg::Default, dialog)
                        }
                        gemtext => {
                            let dialog = Dialog::ack(
                                &self.rect, 
                                &self.cfg, 
                                &format!("you've selected {:?}", gemtext));
                            (TabMsg::Default, dialog)
                        }
                    };
                self.dlg = Some(dialog_tuple);
                return Some(TabMsg::Default)
            } else {
                return None
            }
        } else {
            return None
        }
    }
    pub fn update_cfg(&mut self, rect: &Rect, cfg: &Config) {
        self.cfg = cfg.clone();
        self.rect = rect.clone();
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
pub struct Typer {
    cursor_text: CursorText,
}
impl Typer {
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Enter => {
                Some(InputMsg::Text(self.cursor_text.get_text()))
            }
            KeyCode::Left => {
                match self.cursor_text.move_left(1) {
                    true  => Some(InputMsg::Default),
                    false => None,
                }
            }
            KeyCode::Right => {
                match self.cursor_text.move_right(1) {
                    true  => Some(InputMsg::Default),
                    false => None,
                }
            }
            KeyCode::Delete => {
                match self.cursor_text.delete() {
                    true  => Some(InputMsg::Default),
                    false => None,
                }
            }
            KeyCode::Backspace => {
                match self.cursor_text.backspace() {
                    true  => Some(InputMsg::Default),
                    false => None,
                }
            }
            KeyCode::Char(c) => {
                self.cursor_text.insert(*c);
                Some(InputMsg::Default)
            }
            _ => { 
                None
            }
        }
    }
}
#[derive(Clone, Debug)]
pub enum InputType {
    Ack(char),
    Ask(char, char),
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
                            true  => Some(InputMsg::Default),
                            false => None,
                        }
                    }
                    KeyCode::Right => {
                        match cursortext.move_right(1) {
                            true  => Some(InputMsg::Default),
                            false => None,
                        }
                    }
                    KeyCode::Delete => {
                        match cursortext.delete() {
                            true  => Some(InputMsg::Default),
                            false => None,
                        }
                    }
                    KeyCode::Backspace => {
                        match cursortext.backspace() {
                            true  => Some(InputMsg::Default),
                            false => None,
                        }
                    }
                    KeyCode::Char(c) => {
                        cursortext.insert(*c);
                        Some(InputMsg::Default)
                    }
                    _ => { 
                        None
                    }
                }
            }
            InputType::Ack(ack) => {
                match keycode {
                    KeyCode::Char(c) => {
                        if ack ==  c {
                            return Some(InputMsg::Ack)
                        } else {
                            return None
                        }
                    }
                    _ => None,
                }
            }
            InputType::Ask(yes, no) => {
                match keycode {
                    KeyCode::Char(c) => {
                        if yes ==  c {
                            return Some(InputMsg::Yes)
                        } else if no == c {
                            return Some(InputMsg::No)
                        } else {
                            return None
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
    pub fn text(rect: &Rect, cfg: &Config, prompt: &str) -> Self {
        Self {
            rect:       rect.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Text(
                CursorText::new(rect, "", cfg.colors.get_dialog())),
        }
    }
    pub fn ack(rect: &Rect, cfg: &Config, prompt: &str) -> Self {
        Self {
            rect:       rect.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Ack(cfg.keys.dialog.ack),
        }
    }
    pub fn ask(rect: &Rect, cfg: &Config, prompt: &str ) -> Self {
        Self {
            rect:       rect.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Ask( cfg.keys.dialog.yes, 
                                        cfg.keys.dialog.no  ),
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(MoveTo(self.rect.x, self.rect.y + 4))?
            .queue(Print(&self.prompt))?;
        match &self.input_type {
            InputType::Ack(ack) => {
                stdout
                    .queue(MoveTo(self.rect.x, self.rect.y + 8))?
                    .queue(Print(&format!("|{}| acknowledge", ack)))?;
            }
            InputType::Ask(yes, no) => {
                stdout
                    .queue(MoveTo(self.rect.x, self.rect.y + 8))?
                    .queue(Print(&format!("|{}| yes |{}| no", yes, no)))?;
            }
            InputType::Text(cursortext) => {
                cursortext.view(self.rect.y + 8, stdout)?;
            }
        }
        Ok(())
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        match &mut self.input_type {
            InputType::Text(cursortext) => {
                cursortext.resize(&self.rect)
            }
            _ => {}
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => 
                Some(InputMsg::Cancel),
            _ => 
                self.input_type.update(keycode)
        }
    }
}
