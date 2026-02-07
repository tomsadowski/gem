// ui

use crate::{
    config::{self, Config, ColorParams},
    gemini::{GemType, GemDoc, Scheme},
    text::{Reader, Editor, DisplayText},
    screen::{Screen, DataScreen},
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
    scr:      Screen,
    dscr:     DataScreen,
    cfg:      Config,
    cfg_path: String,
    bg_color: Color,
    view:     View,
    focus:    Focus,
    msg:      MessageView,
    tabs:     TabView,
} 
impl UI {
    // start with View::Tab
    pub fn new(path: &str, w: u16, h: u16) -> Self {
        let (cfg, cfgmsg) = Self::load_config(path);
        let x_margin = cfg.format.margin;
        let y_margin = cfg.format.margin;
        let x_scroll = cfg.scroll_at;
        let y_scroll = cfg.scroll_at;
        let scr = Screen::origin(w, h).crop_x(x_margin).crop_y(y_margin);
        let dscr = DataScreen::new(&scr, x_scroll, y_scroll);
        let mut view = View::Tab;
        let mut msgview = MessageView::init(&dscr, &cfg);
        if let ViewMsg::Msg(msg) = cfgmsg {
            msgview.push(&msg, &cfg);
            view = View::Msg;
        };
        let (tabview, tabmsg) = TabView::init(&dscr, &cfg);
        if let ViewMsg::Msg(msg) = tabmsg {
            msgview.push(&msg, &cfg);
            view = View::Msg;
        };
        Self {
            scr:      scr,
            dscr:     dscr,
            bg_color: cfg.colors.get_background(),
            cfg:      cfg,
            cfg_path: path.into(),
            focus:    Focus::View(view.clone()),
            view:     view,
            tabs:     tabview,
            msg:      msgview,
        }
    }
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
    // resize all views, maybe do this in parallel?
    fn resize(&mut self, w: u16, h: u16) {
        let x_margin = self.cfg.format.margin;
        let y_margin = self.cfg.format.margin;
        let x_scroll = self.cfg.scroll_at;
        let y_scroll = self.cfg.scroll_at;
        let scr = Screen::origin(w, h).crop_x(x_margin).crop_y(y_margin);
        let dscr = DataScreen::new(&scr, x_scroll, y_scroll);
        self.scr = Screen::origin(w, h);
        self.dscr = dscr;
        self.tabs.resize(&self.dscr, &self.cfg);
        self.msg.resize(&self.dscr);
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
                Some(ViewMsg::Default)
            }
            KeyCode::Char(c) => {
                if c == &self.cfg.keys.tab_view {
                    self.view = View::Tab;
                    self.focus = Focus::View(self.view.clone());
                    Some(ViewMsg::Default)
                } else if c == &self.cfg.keys.msg_view {
                    self.view = View::Msg;
                    self.focus = Focus::View(self.view.clone());
                    Some(ViewMsg::Default)
                } else if c == &self.cfg.keys.load_cfg {
                    self.focus = Focus::View(self.view.clone());
                    Some(ViewMsg::ReloadConfig)
                } else {None}
            } 
            _ => {None}
        }

    }
    // Resize and Control-C is handled here, 
    // otherwise delegate to current view
    pub fn update(&mut self, event: Event) -> bool {
        match event {
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
            Event::Resize(w, h) => {
                self.resize(w, h); 
                true
            }
            Event::Key(
                KeyEvent {
                    code: keycode, 
                    kind: KeyEventKind::Press, ..
                }
            ) => {
                let view_msg = match &self.focus {
                    Focus::Global => self.update_global(&keycode),
                    Focus::View(view) => match view {
                        View::Tab => self.tabs.update(&keycode, &self.cfg),
                        View::Msg => self.msg.update(&keycode, &self.cfg),
                        _ => None,
                    },
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
                        self.msg.push(&s, &self.cfg);
                        true
                    }
                    Some(_) => true,
                    None => false
                } 
            }
            _ => false,
        }
    }
    fn update_cfg(&mut self, cfg: Config) {
        self.cfg = cfg;
        self.bg_color = self.cfg.colors.get_background();
        self.tabs.update_cfg(&self.dscr, &self.cfg);
    }
    // no need to derive PartialEq for View
    pub fn is_quit(&self) -> bool {
        match self.view {View::Quit => true, _ => false}
    }
} 
pub struct MessageView {
    dscr:     DataScreen,
    messages: Vec<String>,
    reader:   Reader,
}
impl MessageView {
    pub fn push(&mut self, msg: &str, cfg: &Config) {
        self.messages.push(msg.into());
        self.reader = Reader::one_color( 
            &self.dscr,
            &self.messages, 
            cfg.colors.get_dialog());
    }
    pub fn init(dscr: &DataScreen, cfg: &Config) -> Self {
        let reader = Reader::one_color( 
            dscr, 
            &vec![],
            cfg.colors.get_dialog());
        Self {
            dscr:       dscr.clone(),
            reader:     reader,
            messages:   vec![],
        }
    }
    // resize reader
    pub fn resize(&mut self, dscr: &DataScreen) {
        self.dscr = dscr.clone();
        self.reader.resize(&dscr);
    }
    // show dialog if there's a dialog, otherwise show reader
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        self.reader.view(stdout)
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode, cfg: &Config) 
        -> Option<ViewMsg> 
    {
        if let KeyCode::Char(c) = keycode {
            if c == &cfg.keys.global {
                Some(ViewMsg::Global)
            }
            else if c == &cfg.keys.tab.move_left {
                self.reader.move_left(1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_right {
                self.reader.move_right(1)
                    .then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_down {
                self.reader.move_down(1).then_some(ViewMsg::Default)
            }
            else if c == &cfg.keys.tab.move_up {
                self.reader.move_up(1).then_some(ViewMsg::Default)
            } else {None}
        } else {None}
    }
}
pub struct TabView {
    dscr:     DataScreen,
    hdr_text: DisplayText,
    hdr_line: DisplayText,
    tabs:     Vec<Tab>,
    idx:      usize,
}
impl TabView {
    pub fn init(dscr: &DataScreen, cfg: &Config) -> (Self, ViewMsg) {
        let (tab, msg) = Tab::init(dscr, &cfg.init_url, cfg);
        let tabview = Self {
            hdr_text: 
                Self::get_hdr_text
                    (dscr.outer.x.len16(), &cfg, 0, 1, &cfg.init_url),
            hdr_line: Self::get_hdr_line(dscr.outer.x.len16(), &cfg),
            dscr:     dscr.clone(),
            tabs:     vec![tab],
            idx:      0,
        };
        (tabview, msg)
    }
    // adjust length of banner line, resize all tabs
    pub fn resize(&mut self, dscr: &DataScreen, cfg: &Config) {
        self.dscr     = dscr.clone();
        self.hdr_line = Self::get_hdr_line(self.dscr.outer.x.len16(), &cfg);
        for tab in self.tabs.iter_mut() {
            tab.resize(&self.dscr);
        }
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode, cfg: &Config) 
        -> Option<ViewMsg> 
    {
        let response = self.tabs[self.idx].update(keycode, cfg);
        if let Some(msg) = response {
            let mut viewmsg: Option<ViewMsg> = Some(ViewMsg::Default);
            match msg {
                TabMsg::ViewMsg(m) => {
                    viewmsg = Some(m);
                }
                TabMsg::Go(url) => {
                    let (tab, m) = Tab::init(&self.dscr, &url, cfg);
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
                    if self.idx == 0 {
                        self.idx = self.tabs.len() - 1;
                    } else {
                        self.idx -= 1;
                    }
                }
                TabMsg::CycleRight => {
                    if self.idx == self.tabs.len() - 1 {
                        self.idx = 0;
                    } else {
                        self.idx += 1;
                    }
                }
                _ => {},
            }
            let len = self.tabs.len();
            let url = &self.tabs[self.idx].url;
            self.hdr_text = 
                Self::get_hdr_text(
                    self.dscr.outer.x.len16(), 
                    cfg, 
                    self.idx, 
                    len, 
                    &url);
            self.hdr_line = Self::get_hdr_line(self.dscr.outer.x.len16(), cfg);
            viewmsg
        } else {
            None
        }
    }
    // display banner and reader
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(MoveTo(self.dscr.outer.x.start, 0))?
            .queue(SetForegroundColor(self.hdr_text.color))?
            .queue(Print(&self.hdr_text.text))?
            .queue(MoveTo(self.dscr.outer.x.start, 1))?
            .queue(SetForegroundColor(self.hdr_line.color))?
            .queue(Print(&self.hdr_line.text))?;
        self.tabs[self.idx].view(stdout)
    }
    pub fn update_cfg(&mut self, dscr: &DataScreen, cfg: &Config) {
        self.dscr     = dscr.clone();
        self.hdr_text = 
            Self::get_hdr_text(
                self.dscr.outer.x.len16(), 
                cfg, 
                self.idx, 
                self.tabs.len(), 
                &self.tabs[self.idx].url);
        self.hdr_line = Self::get_hdr_line(self.dscr.outer.x.len16(), cfg);
        for tab in self.tabs.iter_mut() {
            tab.update_cfg(&self.dscr, cfg);
        }
    }
    fn get_hdr_text(    w:          u16,
                        cfg:        &Config,
                        idx:        usize, 
                        total_tab:  usize, 
                        path:       &str    ) -> DisplayText 
    {
        let text = &format!("{}/{}: {}", idx + 1, total_tab, path);
        let width = std::cmp::min(usize::from(w), text.len());
        DisplayText::new(
                &text[..usize::from(width)],
                cfg.colors.get_banner(),
                false
            )
    }
    fn get_hdr_line(w: u16, cfg: &Config) -> DisplayText {
        DisplayText::new(
                &String::from("-").repeat(usize::from(w)),
                cfg.colors.get_banner(),
                false
            )
    }
}
pub struct Tab {
    dscr:    DataScreen,
    pub url: String,
    pub doc: Option<GemDoc>,
    pub dlg: Option<(TabMsg, Dialog)>,
    pub reader: Reader,
}
impl Tab {
    pub fn init(dscr: &DataScreen, url_str: &str, cfg: &Config) 
        -> (Self, ViewMsg) 
    {
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
        let reader = Self::get_reader(dscr, &doc, cfg);
        let tab = Self {
            url:  String::from(url_str),
            dscr: dscr.clone(),
            dlg:  None,
            reader: reader,
            doc:  doc,
        };
        (tab, msg)
    }
    // resize reader and dialog
    pub fn resize(&mut self, dscr: &DataScreen) {
        self.dscr = dscr.clone();
        self.reader.resize(&dscr);
        if let Some((_, d)) = &mut self.dlg {
            d.resize(&dscr);
        }
    }
    pub fn update(&mut self, keycode: &KeyCode, cfg: &Config) -> Option<TabMsg> {
        // send keycode to dialog if there is a dialog.
        if let Some((m, d)) = &mut self.dlg {
            // process response
            match d.update(keycode) {
                Some(InputMsg::Yes) => {
                    let msg = Some(m.clone());
                    self.dlg = None;
                    msg
                }
                Some(InputMsg::No) => {
                    self.dlg = None;
                    Some(TabMsg::Default)
                }
                Some(InputMsg::Ack) => {
                    let msg = Some(m.clone());
                    self.dlg = None;
                    msg
                }
                Some(InputMsg::Text(text)) => {
                    let msg = if let TabMsg::NewTab = m {
                        Some(TabMsg::Go(text))
                    } else {
                        Some(TabMsg::Default)
                    };
                    self.dlg = None;
                    msg
                }
                Some(InputMsg::Cancel) => {
                    self.dlg = None;
                    Some(TabMsg::Default)
                }
                Some(_) => {
                    Some(TabMsg::Default)
                }
               _ => None
            }
        }
        // there is no dialog, process keycode
        else if let KeyCode::Char(c) = keycode {
            if c == &cfg.keys.global {
                Some(TabMsg::ViewMsg(ViewMsg::Global))
            }
            else if c == &cfg.keys.tab.move_down {
                self.reader.move_down(1)
                    .then_some(TabMsg::Default)
            }
            else if c == &cfg.keys.tab.move_up {
                self.reader.move_up(1)
                    .then_some(TabMsg::Default)
            }
            else if c == &cfg.keys.tab.move_left {
                self.reader.move_left(1)
                    .then_some(TabMsg::Default)
            }
            else if c == &cfg.keys.tab.move_right {
                self.reader.move_right(1)
                    .then_some(TabMsg::Default)
            }
            else if c == &cfg.keys.tab.cycle_left {
                Some(TabMsg::CycleLeft)
            }
            else if c == &cfg.keys.tab.cycle_right {
                Some(TabMsg::CycleRight)
            }
            // make a dialog
            else if c == &cfg.keys.tab.delete_tab {
                let dialog = Dialog::ask(
                    &self.dscr, cfg, "Delete current tab?");
                self.dlg = Some((TabMsg::DeleteMe, dialog));
                Some(TabMsg::Default)
            }
            else if c == &cfg.keys.tab.new_tab {
                let dialog = Dialog::text(
                    &self.dscr, cfg, "enter path: ");
                self.dlg = Some((TabMsg::NewTab, dialog));
                Some(TabMsg::Default)
            }
            else if c == &cfg.keys.tab.inspect {
                let gemtype = match &self.doc {
                    Some(doc) => 
                        doc.doc[self.reader.select().0].0.clone(),
                    None => GemType::Text,
                };
                let dialog_tuple = match gemtype {
                    GemType::Link(Scheme::Gemini, url) => {
                        let msg = &format!("go to {}?", url);
                        let dialog = Dialog::ask(&self.dscr, cfg, &msg);
                        (TabMsg::Go(url.into()), dialog)
                    }
                    GemType::Link(_, url) => {
                        let msg = format!("Protocol {} not yet supported", url);
                        let dialog = Dialog::ack(&self.dscr, cfg, &msg);
                        (TabMsg::Default, dialog)
                    }
                    gemtext => {
                        let msg = format!("you've selected {:?}", gemtext);
                        let dialog = Dialog::ack(&self.dscr, cfg, &msg);
                        (TabMsg::Default, dialog)
                    }
                };
                self.dlg = Some(dialog_tuple);
                Some(TabMsg::Default)
            } else {None}
        } else {None}
    }
    pub fn update_cfg(&mut self, dscr: &DataScreen, cfg: &Config) {
        self.dscr = dscr.clone();
        self.reader = Self::get_reader(&self.dscr, &self.doc, cfg);
    }
    // show dialog if there's a dialog, otherwise show reader
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        if let Some((_, d)) = &self.dlg {
            d.view(stdout)
        } else {
            self.reader.view(stdout)
        }
    }
    fn get_reader(dscr: &DataScreen, doc: &Option<GemDoc>, cfg: &Config) 
        -> Reader 
    {
        let colored_text = 
            if let Some(gemdoc) = &doc {
                cfg.colors.from_gem_doc(&gemdoc) 
            } else {
                vec![
                    cfg.colors.from_gem_type(
                        &GemType::Text, 
                        "Nothing to display")]
            };
        Reader::new(dscr, &colored_text)
    }
}
#[derive(Clone, Debug)]
pub enum InputType {
    Ack(char),
    Ask(char, char),
    Text(Editor),
}
impl InputType {
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match self {
            InputType::Text(editor) => {
                match keycode {
                    KeyCode::Enter => {
                        Some(InputMsg::Text(editor.get_text()))
                    }
                    KeyCode::Left => {
                        editor.move_left(1).then_some(InputMsg::Default)
                    }
                    KeyCode::Right => {
                        editor.move_right(1).then_some(InputMsg::Default)
                    }
                    KeyCode::Delete => {
                        editor.delete().then_some(InputMsg::Default)
                    }
                    KeyCode::Backspace => {
                        editor.backspace().then_some(InputMsg::Default)
                    }
                    KeyCode::Char(c) => {
                        editor.insert(*c);
                        Some(InputMsg::Default)
                    }
                    _ => None
                }
            }
            InputType::Ack(ack) => {
                match keycode {
                    KeyCode::Char(c) => {
                        (ack ==  c).then_some(InputMsg::Ack)
                    }
                    _ => None,
                }
            }
            InputType::Ask(yes, no) => {
                match keycode {
                    KeyCode::Char(c) => {
                        if yes ==  c {
                            Some(InputMsg::Yes)
                        } else if no == c {
                            Some(InputMsg::No)
                        } else {
                            None
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
    dscr:       DataScreen,
    prompt:     String,
    input_type: InputType,
}
impl Dialog {
    pub fn text(dscr: &DataScreen, cfg: &Config, prompt: &str) -> Self {
        Self {
            dscr:       dscr.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Text(
                Editor::new(dscr, "", cfg.colors.get_dialog())),
        }
    }
    pub fn ack(dscr: &DataScreen, cfg: &Config, prompt: &str) -> Self {
        Self {
            dscr:       dscr.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Ack(cfg.keys.dialog.ack),
        }
    }
    pub fn ask(dscr: &DataScreen, cfg: &Config, prompt: &str ) -> Self {
        Self {
            dscr:       dscr.clone(),
            prompt:     prompt.into(), 
            input_type: InputType::Ask( cfg.keys.dialog.yes, 
                                        cfg.keys.dialog.no  ),
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(MoveTo(
                    self.dscr.outer.x.start, 
                    self.dscr.outer.y.start + 4))?
            .queue(Print(&self.prompt))?;
        match &self.input_type {
            InputType::Ack(ack) => {
                stdout
                    .queue(MoveTo(
                            self.dscr.outer.x.start, 
                            self.dscr.outer.y.start + 8))?
                    .queue(Print(&format!("|{}| acknowledge", ack)))?;
            }
            InputType::Ask(yes, no) => {
                stdout
                    .queue(MoveTo(
                            self.dscr.outer.x.start, 
                            self.dscr.outer.y.start + 8))?
                    .queue(Print(&format!("|{}| yes |{}| no", yes, no)))?;
            }
            InputType::Text(editor) => {
                editor.view(stdout)?;
            }
        }
        Ok(())
    }
    pub fn resize(&mut self, dscr: &DataScreen) {
        self.dscr = dscr.clone();
        match &mut self.input_type {
            InputType::Text(editor) => {
                editor.resize(&self.dscr)
            }
            _ => {}
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<InputMsg> {
        match keycode {
            KeyCode::Esc => Some(InputMsg::Cancel),
            _ => self.input_type.update(keycode)
        }
    }
}
