// config

use serde::Deserialize;
use crate::{
    gemini::{GemType, GemDoc},
    util::{Rect},
    widget::{Selector, ColoredText},
    dialog::{Dialog, InputType, InputMsg},
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    event::{KeyCode},
    style::{self, Color},
};

#[derive(Deserialize, Debug, Clone)]
pub struct Colors {
    pub background: (u8, u8, u8),
    pub ui:         (u8, u8, u8),
    pub text:       (u8, u8, u8),
    pub heading1:   (u8, u8, u8),
    pub heading2:   (u8, u8, u8),
    pub heading3:   (u8, u8, u8),
    pub link:       (u8, u8, u8),
    pub badlink:    (u8, u8, u8),
    pub quote:      (u8, u8, u8),
    pub listitem:   (u8, u8, u8),
    pub preformat:  (u8, u8, u8),
}
#[derive(Deserialize, Debug, Clone)]
pub struct Keys {
    pub yes: char,
    pub no: char,
    pub move_cursor_up: char,
    pub move_cursor_down: char,
    pub cycle_to_left_tab: char,
    pub cycle_to_right_tab: char,
    pub inspect_under_cursor: char,
    pub delete_current_tab: char,
    pub new_tab: char,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Format {
    pub margin:     u8,
    pub listbullet: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub init_url:  String,
    pub scroll_at: u8,
    pub colors:    Colors,
    pub keys:      Keys,
    pub format:    Format,
}
impl Config {
    pub fn new(text: &str) -> Self {
        toml::from_str(text).unwrap()
    }
}
pub fn getbackground(config: &Colors) -> Color {
    Color::Rgb {
        r: config.background.0,
        g: config.background.1,
        b: config.background.2,
    }
}
pub fn getvec(vec: &Vec<(GemType, String)>, config: &Colors) 
    -> Vec<ColoredText>
{
    vec
        .iter()
        .map(|(g, s)| getcoloredgem(g, &s, config))
        .collect()
}
pub fn getcoloredgem(gem: &GemType, 
                     text: &str, 
                     config: &Colors) -> ColoredText {
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
