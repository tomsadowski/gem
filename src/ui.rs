// ui

use crate::{
    config::{self, Config, Colors},
    gemini::{GemType, GemDoc, Scheme},
    widget::{Rect, Pager, CursorText, ColoredText},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    style::{self, Color},
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use std::{
    io::{self, Stdout, Write},
};
use url::Url;

// view currently in use
#[derive(Debug)]
pub enum View {
    Tab,
    Quit,
}
// coordinates activities between views
pub struct UI {
    rect:    Rect,
    config:  Config,
    bgcolor: Color,
    view:    View,
    tabs:    TabServer,
} 
impl UI {
    // start with View::Tab
    pub fn new(config: &Config, w: u16, h: u16) -> Self {
        let rect = Rect {x: 0, y: 0, w: w, h: h};
        Self {
            tabs:    TabServer::new(&rect, config),
            rect:    rect,
            config:  config.clone(),
            view:    View::Tab,
            bgcolor: config.colors.get_background(),
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
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(style::SetBackgroundColor(self.bgcolor))?;
        match &self.view {
            View::Tab => self.tabs.view(stdout),
            _ => Ok(()),
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
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press, ..
            }) => {
                self.view = View::Quit;
                true
            }
            Event::Key(KeyEvent {
                code: keycode, 
                kind: KeyEventKind::Press, ..
            }) => 
                match &self.view {
                    View::Tab => self.tabs.update(&keycode),
                    _ => false,
                }
            _ => false,
        }
    }
    // no need to derive PartialEq for View
    pub fn quit(&self) -> bool {
        match self.view {
            View::Quit => true,
            _ => false,
        }
    }
} 
pub struct TabServer {
    rect:        Rect,
    config:      Config,
    banner_text: ColoredText,
    banner_line: ColoredText,
    tabs:        Vec<Tab>,
    cur_index:   usize,
}
impl TabServer {
    pub fn new(r: &Rect, config: &Config) -> Self {
        let rect = Rect {
            x: r.x + config.format.margin as u16, 
            y: r.y + 2, 
            w: r.w - (config.format.margin * 2) as u16,
            h: r.h - 2
        };
        Self {
            rect:        rect.clone(),
            config:      config.clone(),
            tabs:        vec![Tab::new(&rect, &config.init_url, config)],
            cur_index:   0,
            banner_text: Self::get_banner_text(0, 1, &config.init_url),
            banner_line: Self::get_banner_line(rect.w),
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
        self.banner_line = Self::get_banner_line(self.rect.w);
        for d in self.tabs.iter_mut() {
            d.resize(&self.rect);
        }
    }
    // display banner and page
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x, 0))?
            .queue(style::SetForegroundColor(self.banner_text.color))?
            .queue(style::Print(&self.banner_text.text))?
            .queue(cursor::MoveTo(self.rect.x, 1))?
            .queue(style::SetForegroundColor(self.banner_line.color))?
            .queue(style::Print(&self.banner_line.text))?;
        self.tabs[self.cur_index].view(stdout)
    }
    // send keycode to current tab and process response
    pub fn update(&mut self, keycode: &KeyCode) -> bool {
        match self.tabs[self.cur_index].update(keycode) {
            Some(msg) => {
                match msg {
                    TabMsg::Go(url) => {
                        self.tabs.push(
                            Tab::new(&self.rect, &url, &self.config));
                        self.cur_index = self.tabs.len() - 1;
                    }
                    TabMsg::Open(text) => {
                        self.tabs.push(
                            Tab::new(&self.rect, &text, &self.config));
                        self.cur_index = self.tabs.len() - 1;
                    }
                    TabMsg::DeleteMe => {
                        if self.tabs.len() > 1 {
                            self.tabs.remove(self.cur_index);
                            self.cur_index = self.tabs.len() - 1;
                        }
                    }
                    TabMsg::CycleLeft => {
                        match self.cur_index == 0 {
                            true => 
                                self.cur_index = self.tabs.len() - 1,
                            false => self.cur_index -= 1,
                        }
                    }
                    TabMsg::CycleRight => {
                        match self.cur_index == self.tabs.len() - 1 {
                            true => self.cur_index = 0,
                            false => self.cur_index += 1,
                        }
                    }
                    _ => {},
                }
                let len = self.tabs.len();
                let url = self.tabs[self.cur_index].get_url();
                self.banner_text = 
                    Self::get_banner_text(self.cur_index, len, url);
                self.banner_line = Self::get_banner_line(self.rect.w);
                true
            }
            None => false,
        }
    }
    fn get_banner_text(cur_index: usize, total_tab: usize, url: &str) 
        -> ColoredText 
    {
        ColoredText::white(
            &format!("{}/{}: {}", cur_index + 1, total_tab, url))
    }
    fn get_banner_line(w: u16) -> ColoredText {
        ColoredText::white(&String::from("-").repeat(usize::from(w)))
    }
}
pub struct Tab {
    rect:   Rect,
    config: Config,
    url:    String,
    doc:    Option<GemDoc>,
    dlg:    Option<(TabMsg, Dialog)>,
    page:   Pager,
}
impl Tab {
    pub fn get_url(&self) -> &str {
        &self.url
    }
    pub fn new(rect: &Rect, url_str: &str, config: &Config) 
        -> Self 
    {
        let doc = match Url::parse(url_str) {
            Ok(url) => Some(GemDoc::new(&url)),
            _ => None,
        };
        let page = match &doc {
            Some(gemdoc) => 
                Pager::new(
                        rect, 
                        &config.colors.from_gem_doc(&gemdoc),
                        config.scroll_at),
            None => {
                let colored_text = config.colors
                    .from_gem_type(
                        &GemType::Text, 
                        "Nothing to display");
                Pager::new(
                        rect, 
                        &vec![colored_text],
                        config.scroll_at)
            }
        };
        Self {
            url:    String::from(url_str),
            config: config.clone(),
            rect:   rect.clone(),
            dlg:    None,
            page:   page,
            doc:    doc,
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
    // show dialog if there's a dialog, otherwise show page
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        match &self.dlg {
            Some((_, d)) => d.view(stdout),
            _ => self.page.view(stdout),
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<TabMsg> {
        // send keycode to dialog if there is a dialog
        if let Some((m, d)) = &mut self.dlg {
            match d.update(keycode) {
                Some(InputMsg::Choose(c)) => {
                    let msg = match c == self.config.keys.yes {
                        false => Some(TabMsg::None),
                        true =>  Some(m.clone()),
                    };
                    self.dlg = None;
                    return msg
                }
                Some(InputMsg::Text(text)) => {
                    let msg = match m {
                        TabMsg::NewTab => 
                            Some(TabMsg::Open(text)),
                        _ => Some(TabMsg::None),
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
                    Dialog::choose(
                        &self.rect,
                        "Delete current tab?",
                        vec![(self.config.keys.yes, "yes"),
                             (self.config.keys.no, "no")]);
                self.dlg = Some((TabMsg::DeleteMe, dialog));
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.new_tab {
                let dialog = Dialog::text(&self.rect, "enter path: ");
                self.dlg = Some((TabMsg::NewTab, dialog));
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.inspect_under_cursor {
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
                                vec![(self.config.keys.yes, "yes"), 
                                     (self.config.keys.no, "no")]);
                            (TabMsg::Go(url.to_string()), dialog)
                        }
                        GemType::Link(_, url) => {
                            let dialog = Dialog::choose(
                                &self.rect,
                                &format!("Protocol {} not yet supported", url),
                                vec![(self.config.keys.yes, "acknowledge")]);
                            (TabMsg::Acknowledge, dialog)
                        }
                        gemtext => {
                            let dialog = Dialog::choose(
                                &self.rect,
                                &format!("you've selected {:?}", gemtext),
                                vec![(self.config.keys.yes, "acknowledge")]);
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
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Right => {
                        match cursortext.move_right(1) {
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Delete => {
                        match cursortext.delete() {
                            true => Some(InputMsg::None),
                            false => None,
                        }
                    }
                    KeyCode::Backspace => {
                        match cursortext.backspace() {
                            true => Some(InputMsg::None),
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
                            true => Some(InputMsg::Choose(*c)),
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
    pub fn choose(rect: &Rect, prompt: &str, choose: Vec<(char, &str)>) 
        -> Self
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
            .queue(cursor::MoveTo(self.rect.x, self.rect.y + 4))?
            .queue(style::Print(self.prompt.as_str()))?;
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
            KeyCode::Esc => Some(InputMsg::Cancel),
            _ => self.input_type.update(keycode)
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
