// gem/src/tab
use crate::{
    util,
    config::{Config},
    gemini::{self, Scheme, GemTextData, Status},
    widget::{Selector, Rect},
    dialog::{Dialog, InputType, DialogMsg},
};
use crossterm::{
    event::{KeyCode},
};
use std::{
    io::{self, Stdout},
};
use url::Url;

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
    pub url: Url,
    rect: Rect,
    config: Config,
    dlgstack: Vec<Dialog<TabMsg>>,
    page: Selector<GemTextData>,
}
impl Tab {
    pub fn new(rect: &Rect, url: &Url, config: &Config) -> Self {
        let (stat_str, text_str) = util::get_data(url).unwrap();
        let gemtext = match gemini::parse_status(&stat_str) {
            Ok((Status::Success, _)) => 
                gemini::parse_doc(text_str.lines().collect()).unwrap(),
            Ok((status, text)) => 
                vec![(
                    GemTextData::Text, 
                    format!("status reply: {:?} {}", status, text)
                )],
            Err(s) => 
                vec![(
                    GemTextData::Text, 
                    format!("{}", s)
                )],
        };
        Self {
            config: config.clone(),
            rect: rect.clone(),
            url: url.clone(),
            dlgstack: vec![],
            page: Selector::new(rect, gemtext, true),
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
    // show dialog if there's a dialog, otherwise show page
    pub fn view(&self, stdout: &Stdout) -> io::Result<()> {
        match self.dlgstack.last() {
            Some(d) => d.view(stdout),
            _ => self.page.view(stdout),
        }
    }
    pub fn update(&mut self, keycode: &KeyCode) -> Option<TabMsg> {
        // send keycode to dialog if there is a dialog
        if let Some(d) = self.dlgstack.last_mut() {
            match d.update(keycode) {
                Some(DialogMsg::Submit(a)) => {
                    let msg = 
                        match &d.input {
                            InputType::Choose((c, _)) => {
                                match c == &self.config.keys.yes {
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
            if c == &self.config.keys.move_cursor_down {
                self.page.cursor.movedown(1);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.move_cursor_up {
                self.page.cursor.moveup(1);
                return Some(TabMsg::None)
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
                    Dialog::new(
                        &self.rect,
                        TabMsg::DeleteMe,
                        InputType::Choose((
                            'n', 
                            vec![
                                (self.config.keys.yes, 
                                 String::from("yes")),
                                (self.config.keys.no, 
                                 String::from("no"))
                            ])),
                        "Delete current tab?");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.new_tab {
                let dialog = 
                    Dialog::new(
                        &self.rect,
                        TabMsg::Go(String::from("")),
                        InputType::Input(String::from("")),
                        "enter path: ");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.inspect_under_cursor {
                let dialog = match self.page.selectundercursor() {
                    GemTextData::Link(Scheme::Relative(l)) => 
                        Dialog::new(
                            &self.rect,
                            TabMsg::Go(self.url.join(l).unwrap().to_string()),
                            InputType::Choose((
                                'n', 
                                vec![
                                    (self.config.keys.yes, 
                                     String::from("yes")), 
                                    (self.config.keys.no, 
                                     String::from("no"))
                                ])),
                            &format!("go to {}?", l)),
                    GemTextData::Link(Scheme::Gemini(l)) => 
                        Dialog::new(
                            &self.rect,
                            TabMsg::Go(l.to_string()),
                            InputType::Choose((
                                    'n', 
                                    vec![
                                        (self.config.keys.yes, 
                                         String::from("yes")), 
                                        (self.config.keys.no, 
                                         String::from("no"))
                                    ])),
                            &format!("go to {}?", l)),
                    gemtext => 
                        Dialog::new(
                            &self.rect,
                            TabMsg::None,
                            InputType::None,
                            &format!("{:?}", gemtext)),
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
