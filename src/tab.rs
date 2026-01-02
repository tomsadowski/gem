// tab

use crate::{
    config::{Config, GemColors},
    gemini::{GemType, GemDoc},
    geometry::{Rect},
    widget::{Selector, ColoredText},
    dialog::{Dialog, DialogMsg, InputType, InputMsg},
};
use crossterm::{
    event::{KeyCode},
    style::{Color},
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
    Go(Url),
}
pub struct Tab {
    pub doc: GemDoc,
    rect: Rect,
    config: Config,
    dlgstack: Vec<Dialog<TabMsg>>,
    page: Selector,
}
impl Tab {
    pub fn new(rect: &Rect, gemdoc: GemDoc, config: &Config) -> Self {
        Self {
            config: config.clone(),
            rect: rect.clone(),
            dlgstack: vec![],
            page: Selector::new(
                rect, 
                &getvec(&gemdoc.doc, &config.gemcolors)),
            doc: gemdoc,
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
                Some(DialogMsg::Submit(action, submission)) => {
                    let msg = match submission {
                        InputMsg::Choose(c) => {
                            match c == self.config.keys.yes {
                                true => Some(action),
                                false => Some(TabMsg::None),
                            }
                        }
                        InputMsg::Input(_) => Some(TabMsg::None),
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
                        InputType::choose(vec![
                            (self.config.keys.yes, "yes"),
                            (self.config.keys.no, "no")]),
                        "Delete current tab?");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.new_tab {
                let dialog = 
                    Dialog::new(
                        &self.rect,
                        TabMsg::None,
                        InputType::input(),
                        "enter path: ");
                self.dlgstack.push(dialog);
                return Some(TabMsg::None)
            }
            else if c == &self.config.keys.inspect_under_cursor {
                let dialog = 
                    match &self.doc.doc[self.page.selectundercursor().0].0 {
                        GemType::Link(_, url) => 
                            Dialog::new(
                                &self.rect,
                                TabMsg::Go(url.clone()),
                                InputType::choose(vec![
                                    (self.config.keys.yes, "yes"), 
                                    (self.config.keys.no, "no")]),
                                &format!("go to {}?", url)),
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
pub fn getvec(vec: &Vec<(GemType, String)>, 
              config: &GemColors) -> Vec<ColoredText> 
{
    vec
        .iter()
        .map(|(g, s)| getcoloredgem(g, &s, config))
        .collect()
}
pub fn getcoloredgem(gem: &GemType, text: &str, config: &GemColors) -> ColoredText {
    let color = match gem {
        GemType::HeadingOne => 
            Color::Rgb {
                r: config.heading1.0, 
                g: config.heading1.1, 
                b: config.heading1.2},
        GemType::HeadingTwo => 
            Color::Rgb {
                r: config.heading2.0, 
                g: config.heading2.1, 
                b: config.heading2.2},
        GemType::HeadingThree => 
            Color::Rgb {
                r: config.heading3.0, 
                g: config.heading3.1, 
                b: config.heading3.2},
        GemType::Text => 
            Color::Rgb {
                r: config.text.0, 
                g: config.text.1, 
                b: config.text.2},
        GemType::Quote => 
            Color::Rgb {
                r: config.quote.0, 
                g: config.quote.1, 
                b: config.quote.2},
        GemType::ListItem => 
            Color::Rgb {
                r: config.listitem.0, 
                g: config.listitem.1, 
                b: config.listitem.2},
        GemType::PreFormat => 
            Color::Rgb {
                r: config.preformat.0, 
                g: config.preformat.1, 
                b: config.preformat.2},
        GemType::Link(_, _) => 
            Color::Rgb {
                r: config.link.0, 
                g: config.link.1, 
                b: config.link.2},
        GemType::BadLink(_) => 
            Color::Rgb {
                r: config.badlink.0, 
                g: config.badlink.1, 
                b: config.badlink.2},
    };
    ColoredText::new(text, color)
}
