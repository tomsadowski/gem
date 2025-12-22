// gem/src/ui
// joins backend and frontend

use crate::{
    config::{Config, Keys},
    gemini::{self, Scheme, GemTextData},
    widget::{Selector, Dialog, InputType, DialogMsg, GetColors, Rect},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
    style::{self, Colors, Color}
};
use std::{
    io::{self, Stdout, Write},
};
use url::{
    Url
};

// coordinates activities between views
#[derive(Clone, Debug)]
pub struct UI {
    rect: Rect,
    view: View,
    tabs: TabMgr,
} 
impl UI {
    // default view is View::Tab
    pub fn new(url: &str, config: &Config, w: u16, h: u16) -> Self {
        let rect = Rect::new(0, 0, w, h);
        Self {
            tabs: TabMgr::new(&rect, config, url),
            rect: rect,
            view: View::Tab,
        }
    }
    // resize all views, maybe do this in parallel?
    fn resize(&mut self, w: u16, h: u16) {
        self.rect = Rect::new(0, 0, w, h);
        self.tabs.resize(&self.rect);
    }
    // display the current view
    pub fn view(&self, mut stdout: &std::io::Stdout) -> io::Result<()> {
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
                kind: KeyEventKind::Press, 
                ..
            }) => {
                self.view = View::Quit;
                true
            }
            Event::Key(KeyEvent {
                code: keycode, 
                kind: KeyEventKind::Press, 
                ..
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
impl GetColors for GemTextData {
    fn getcolors(&self) -> Colors {
        match self {
            Self::HeadingOne => Colors::new(
                Color::Rgb {r: 225, g: 180, b: 105},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::HeadingTwo => Colors::new(
                Color::Rgb {r: 225, g: 180, b: 105},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::HeadingThree => Colors::new(
                Color::Rgb {r: 225, g: 180, b: 105},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::Text => Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::Quote => Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::ListItem => Colors::new(
                Color::Rgb {r: 180, g: 180, b: 180},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::PreFormat => Colors::new(
                Color::Rgb {r: 80 , g: 180, b: 80 },
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
            Self::Link(_) => Colors::new(
                Color::Rgb {r: 105, g: 180, b: 225},
                Color::Rgb {r: 0, g: 0, b: 0},
            ),
        } 
    }
}
#[derive(Clone, Debug)]
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
    pub fn new(rect: &Rect, config: &Config, p: &str) -> Self {
        let rect = Rect::new(rect.x, rect.y + 2, rect.w, rect.h - 1);
        let url = Url::parse(p).unwrap();
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
                    TabMsg::Msg(ViewMsg::Go(p)) => {
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
                let path = &self.tabs[self.curindex].path;
                self.bannerstr = Self::bannerstr(self.curindex, len, path);
                self.bannerline = Self::bannerline(self.rect.w);
                true
            }
            None => false,
        }
    }
    fn bannerstr(curindex: usize, totaltab: usize, path: &Url) -> String {
        format!("{}/{}: {}", curindex + 1, totaltab, path)
    }
    fn bannerline(w: u16) -> String {
        String::from("-").repeat(usize::from(w))
    }
}
#[derive(Clone, Debug)]
pub enum Action {
    None,
    GoTo,
    DeleteMe,
    Go(String),
}
// view currently in use
#[derive(Clone, Debug)]
pub enum View {
    Tab,
    Quit,
}
// message returned from a view's update method
#[derive(Clone, Debug)]
pub enum ViewMsg {
    None,
    Go(String),
}
#[derive(Clone, Debug)]
pub enum TabMsg {
    CycleLeft,
    CycleRight,
    DeleteMe,
    Msg(ViewMsg),
}
#[derive(Clone, Debug)]
pub struct Tab {
    rect: Rect,
    pub path: Url,
    dlgstack: Vec<Dialog<Action>>,
    page: Selector<GemTextData>,
}
impl Tab {
    pub fn new(rect: &Rect, path: &Url) -> Self {
        let src = gemini::get_data(path).unwrap();
        let text = gemini::parse_doc(src.1.lines().collect()).unwrap();
        Self {
            rect: rect.clone(),
            path: path.clone(),
            dlgstack: vec![],
            page: Selector::new(rect, text, true),
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
                Some(DialogMsg::Submit) => {
                    let msg = match (&d.action, &d.input) {
                        (Action::Go(p), InputType::Choose((c, _))) => {
                            match c {
                                'y' => 
                                    Some(TabMsg::Msg(ViewMsg::Go(p.clone()))),
                                _ => 
                                    Some(TabMsg::Msg(ViewMsg::None)),
                            }
                        }
                        (Action::GoTo, InputType::Input(v)) => {
                            Some(TabMsg::Msg(ViewMsg::Go(v.clone())))
                        }
                        (Action::DeleteMe, InputType::Choose((c, _))) => {
                            match c {
                                'y' => Some(TabMsg::DeleteMe),
                                _ => Some(TabMsg::Msg(ViewMsg::None)),
                            }
                        }
                        (_, _) => 
                            Some(TabMsg::Msg(ViewMsg::None)),
                    };
                    self.dlgstack.pop();
                    return msg
                }
                Some(DialogMsg::Cancel) => {
                    self.dlgstack.pop();
                    return Some(TabMsg::Msg(ViewMsg::None))
                }
                Some(_) => {
                    return Some(TabMsg::Msg(ViewMsg::None))
                }
               _ => return None
            }
        }
        // there is no dialog, process keycode here
        if let KeyCode::Char(c) = keycode {
            if c == &keys.delete_current_tab {
                let dialog = Dialog::new(
                    &self.rect,
                    Action::DeleteMe,
                    InputType::Choose(('n', vec![
                        ('y', String::from("yes")), 
                        ('n', String::from("no"))])),
                    "Delete current tab?");
                self.dlgstack.push(dialog);
                return Some(TabMsg::Msg(ViewMsg::None))
            }
            else if c == &keys.new_tab {
                let dialog = Dialog::new(
                    &self.rect,
                    Action::GoTo,
                    InputType::Input(String::from("")),
                    "enter path: ");
                self.dlgstack.push(dialog);
                return Some(TabMsg::Msg(ViewMsg::None))
            }
            else if c == &keys.move_cursor_down {
                self.page.cursor.movedown(1);
                return Some(TabMsg::Msg(ViewMsg::None))
            }
            else if c == &keys.move_cursor_up {
                self.page.cursor.moveup(1);
                return Some(TabMsg::Msg(ViewMsg::None))
            }
            else if c == &keys.cycle_to_left_tab {
                return Some(TabMsg::CycleLeft)
            }
            else if c == &keys.cycle_to_right_tab {
                return Some(TabMsg::CycleRight)
            }
            // make a dialog
            else if c == &keys.inspect_under_cursor {
                let dialog = match self.page.selectundercursor() {
                    GemTextData::Link(Scheme::Relative(l)) => Dialog::new(
                        &self.rect,
                        Action::Go(self.path.join(l).unwrap().to_string()),
                        InputType::Choose(('n', vec![
                            (keys.yes, String::from("yes")), 
                            (keys.no, String::from("no"))])),
                        &format!("go to {}?", l)),
                    GemTextData::Link(Scheme::Gemini(l)) => Dialog::new(
                        &self.rect,
                        Action::Go(l.to_string()),
                        InputType::Choose(('n', vec![
                            (keys.yes, String::from("yes")), 
                            (keys.no, String::from("no"))])),
                        &format!("go to {}?", l)),
                    _ => Dialog::new(
                        &self.rect,
                        Action::None,
                        InputType::None,
                        "You've selected some text. "),
                };
                self.dlgstack.push(dialog);
                return Some(TabMsg::Msg(ViewMsg::None))
            }
            return None
        } else {return None}
    }
}
