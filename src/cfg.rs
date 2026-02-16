// src/cfg.rs

use crate::{
    gem::{GemDoc, GemType},
    text::{Text},
};
use crossterm::{
    style::{Color},
};
use std::fs;
use serde::Deserialize;

// return default config if error
pub fn load_config(path: &str) -> Config {
    fs::read_to_string(path)
        .ok().map(|txt| Config::parse(&txt).ok())
        .flatten().unwrap_or(Config::default())
}
#[derive(Deserialize)]
pub struct Config {
    pub init_url:  String,
    pub scroll_at: u16,
    pub colors:    ColorParams,
    pub keys:      KeyParams,
    pub format:    FormatParams,
} 
impl Config {
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }
    pub fn parse_or_default(text: &str) -> Self {
        toml::from_str(text).unwrap_or(Self::default())
    }
    pub fn default() -> Self {
        Self {
            init_url: "gemini://datapulp.smol.pub/".into(),
            scroll_at: 3,
            colors:    ColorParams::default(),
            keys:      KeyParams::default(),
            format:    FormatParams::default(),
        }
    }
}
#[derive(Deserialize)]
pub struct KeyParams {
    pub global:     char,
    pub load_cfg:   char,
    pub msg_view:   char,
    pub tab_view:   char,
    pub dialog:     DialogKeyParams,
    pub tab:        TabKeyParams,
} 
impl KeyParams {
    pub fn default() -> Self {
        Self {
            global:     'g',
            load_cfg:   'c',
            msg_view:   'm',
            tab_view:   't',
            dialog:     DialogKeyParams::default(),
            tab:        TabKeyParams::default(),
        }
    }
}
#[derive(Deserialize)]
pub struct TabKeyParams {
    pub move_up:      char,
    pub move_down:    char,
    pub move_left:    char,
    pub move_right:   char,
    pub cycle_left:   char,
    pub cycle_right:  char,
    pub inspect:      char,
    pub delete_tab:   char,
    pub new_tab:      char,
} 
impl TabKeyParams {
    pub fn default() -> Self {
        Self {
            move_up:      'o',
            move_down:    'i',
            move_left:    'e',
            move_right:   'n',
            cycle_left:   'E',
            cycle_right:  'N',
            inspect:      'w',
            delete_tab:   'v',
            new_tab:      'p',
        }
    }
}
#[derive(Deserialize)]
pub struct DialogKeyParams {
    pub ack: char, 
    pub yes: char, 
    pub no: char
} 
impl DialogKeyParams {
    pub fn default() -> Self {
        Self {ack: 'y', yes: 'y', no: 'n'}
    }
}
#[derive(Deserialize)]
pub struct FormatParams {
    pub margin:      u16,
    pub list_prefix: String,
    pub heading1:    (u8, u8),
    pub heading2:    (u8, u8),
    pub heading3:    (u8, u8),
} 
impl FormatParams {
    pub fn default() -> Self {
        Self {
            margin:      2,
            list_prefix: "- ".into(),
            heading1:    (3, 2),
            heading2:    (2, 1),
            heading3:    (1, 0),
        }
    }
}
#[derive(Deserialize)]
pub struct ColorParams {
    pub background: (u8, u8, u8),
    pub banner:     (u8, u8, u8),
    pub dialog:     (u8, u8, u8),
    pub text:       (u8, u8, u8),
    pub heading1:   (u8, u8, u8),
    pub heading2:   (u8, u8, u8),
    pub heading3:   (u8, u8, u8),
    pub link:       (u8, u8, u8),
    pub badlink:    (u8, u8, u8),
    pub quote:      (u8, u8, u8),
    pub list:       (u8, u8, u8),
    pub preformat:  (u8, u8, u8),
} 
impl ColorParams {
    pub fn default() -> Self {
        Self {
            background: (205, 205, 205),
            dialog:     (  0,   0,   0),
            banner:     (  0,   0,   0),
            text:       (  0,   0,   0),
            heading1:   (  0,   0,   0),
            heading2:   (  0,   0,   0),
            heading3:   (  0,   0,   0),
            link:       (  0,   0,   0),
            badlink:    (  0,   0,   0),
            quote:      (  0,   0,   0),
            list:       (  0,   0,   0),
            preformat:  (  0,   0,   0),
        }
    }
    pub fn get_banner(&self) -> Color {
        let (r, g, b) = self.banner; Color::Rgb {r, g, b}
    }
    pub fn get_dialog(&self) -> Color {
        let (r, g, b) = self.dialog; Color::Rgb {r, g, b}
    }
    pub fn get_background(&self) -> Color {
        let (r, g, b) = self.background; Color::Rgb {r, g, b}
    }
    pub fn from_gem_doc(&self, doc: &GemDoc) -> Vec<Text> {
        doc.doc.iter()
            .map(|(gem_type, text)| self.from_gem_type(gem_type, &text))
            .collect()
    }
    pub fn from_gem_type(&self, gem: &GemType, text: &str) -> Text {
        let ((r, g, b), wrap) = match gem {
            GemType::HeadingOne     => (self.heading1, true),
            GemType::HeadingTwo     => (self.heading2, true),
            GemType::HeadingThree   => (self.heading3, true),
            GemType::Text           => (self.text, true),
            GemType::Quote          => (self.quote, true),
            GemType::ListItem       => (self.list, true),
            GemType::PreFormat      => (self.preformat, false),
            GemType::Link(_, _)     => (self.link, true),
            GemType::BadLink(_)     => (self.badlink, true),
        };
        Text::new(text, Color::Rgb {r, g, b}, wrap)
    }
}
