// gem/src/tabs
use crate::{
    config::{Config, Keys},
    gemini::{self, Scheme, GemTextData, Status},
    widget::{Selector, Rect},
    dialog::{Dialog, InputType, DialogMsg}};
use crossterm::{
    QueueableCommand, cursor, terminal,
    event::{KeyCode},
    style::{self, Colors, Color}};
use std::{
    io::{self, Stdout}};
use url::Url;

pub struct TabMgr {
    rect: Rect,
    keys: Keys,
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
impl TabMgr {
    pub fn new(rect: &Rect, config: &Config) -> Self {
        let rect = Rect::new(rect.x, rect.y + 2, rect.w, rect.h - 1);
        let url = Url::parse(&config.init_url).unwrap();
        Self {
            rect: rect.clone(),
            keys: config.keys.clone(),
            tabs: vec![Tab::new(&rect, &url)],
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
        match self.tabs[self.curindex].update(&self.keys, keycode) {
            Some(msg) => {
                match msg {
                    TabMsg::Go(p) => {
                        let url = Url::parse(p.as_str()).unwrap();
                        self.tabs.push(Tab::new(&self.rect, &url));
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
#[derive(Clone, Debug)]
pub enum TabMsg {
    Quit,
    None,
    CycleLeft,
    CycleRight,
    // requires dialog
    DeleteMe,
    Go(String),
}
#[derive(Clone, Debug)]
pub struct Tab {
    rect: Rect,
    pub url: Url,
    dlgstack: Vec<Dialog<TabMsg>>,
    page: Selector<GemTextData>,
}
impl Tab {
    pub fn new(rect: &Rect, url: &Url) -> Self {
        let (stat_str, text_str) = gemini::get_data(url).unwrap();
        let gemtext = match gemini::parse_status(&stat_str) {
            Ok((Status::Success, _)) => 
                gemini::parse_doc(text_str.lines().collect()).unwrap(),
            Ok((_, s)) => 
                vec![(
                    GemTextData::Text, 
                    format!("status reply: {}", s)
                )],
            Err(s) => 
                vec![(
                    GemTextData::Text, 
                    format!("{}", s)
                )],
        };
        Self {
            rect: rect.clone(),
            url: url.clone(),
            dlgstack: vec![],
            page: Selector::new(rect, gemtext, true),
        }
    }
    // show dialog if there's a dialog, otherwise show page
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        match self.dlgstack.last() {
            Some(d) => d.view(stdout),
            _ => self.page.view(stdout),
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
    pub fn update(&mut self, keys: &Keys, keycode: &KeyCode) 
        -> Option<TabMsg> 
    {
        // send keycode to dialog if there is a dialog
        if let Some(d) = self.dlgstack.last_mut() {
            match d.update(keycode) {
                Some(DialogMsg::Submit(a)) => {
                    let msg = match &d.input {
                        InputType::Choose((c, _)) => {
                            match c == &keys.yes {
                                true => Some(a),
                                false => Some(TabMsg::None),
                            }
                        }
                        InputType::Input(v) => {
                            match a {
                                TabMsg::Go(_) => 
                                    Some(TabMsg::Go(v.to_string())),
                                _ => Some(TabMsg::None),
                            }
                        }
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
            if c == &keys.move_cursor_down {
                self.page.cursor.movedown(1);
                return Some(TabMsg::None)
            }
            else if c == &keys.move_cursor_up {
                self.page.cursor.moveup(1);
                return Some(TabMsg::None)
            }
            else if c == &keys.cycle_to_left_tab {
                return Some(TabMsg::CycleLeft)
            }
            else if c == &keys.cycle_to_right_tab {
                return Some(TabMsg::CycleRight)
            }
            // make a dialog
            else if c == &keys.delete_current_tab {
                let dialog = 
                    Dialog::new(
                        &self.rect,
                        TabMsg::DeleteMe,
                        InputType::Choose((
                            'n', 
                            vec![
                                (keys.yes, String::from("yes")),
                                (keys.no, String::from("no"))
                            ])),
                        "Delete current tab?");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &keys.new_tab {
                let dialog = 
                    Dialog::new(
                        &self.rect,
                        TabMsg::Go(String::from("")),
                        InputType::Input(String::from("")),
                        "enter path: ");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &keys.inspect_under_cursor {
                let dialog = match self.page.selectundercursor() {
                    GemTextData::Link(Scheme::Relative(l)) => 
                        Dialog::new(
                            &self.rect,
                            TabMsg::Go(self.url.join(l).unwrap().to_string()),
                            InputType::Choose((
                                'n', 
                                vec![
                                    (keys.yes, String::from("yes")), 
                                    (keys.no, String::from("no"))
                                ])),
                            &format!("go to {}?", l)),
                    GemTextData::Link(Scheme::Gemini(l)) => 
                        Dialog::new(
                            &self.rect,
                            TabMsg::Go(l.to_string()),
                            InputType::Choose((
                                    'n', 
                                    vec![
                                        (keys.yes, String::from("yes")), 
                                        (keys.no, String::from("no"))
                                    ])),
                            &format!("go to {}?", l)),
                    _ => 
                        Dialog::new(
                            &self.rect,
                            TabMsg::None,
                            InputType::None,
                            "You've selected some text. "),
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
