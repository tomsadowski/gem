// config

use crate::{
    gemini::{GemType, GemDoc, Scheme},
    widget::{ColoredText},
};
use crossterm::{
    style::{self, Color},
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub init_url:  String,
    pub scroll_at: u8,
    pub colors:    Colors,
    pub keys:      Keys,
    pub format:    Format,
}
impl Config {
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }
    pub fn parse_or_default(text: &str) -> Self {
        let result = toml::from_str(text);
        match result {
            Ok(cfg) => cfg,
            _ => Self::default(),
        }
    }
    pub fn default() -> Self {
        Self {
            init_url:  "".into(),
            scroll_at: 3,
            colors:    Colors::default(),
            keys:      Keys::default(),
            format:    Format::default(),
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
pub struct Keys {
    pub yes:                    char,
    pub no:                     char,
    pub move_cursor_up:         char,
    pub move_cursor_down:       char,
    pub cycle_to_left_tab:      char,
    pub cycle_to_right_tab:     char,
    pub inspect_under_cursor:   char,
    pub delete_current_tab:     char,
    pub new_tab:                char,
}
impl Keys {
    pub fn default() -> Self {
        Self {
            yes:                    'y',
            no:                     'n',
            move_cursor_up:         'j',
            move_cursor_down:       'k',
            cycle_to_left_tab:      'h',
            cycle_to_right_tab:     'l',
            inspect_under_cursor:   'i',
            delete_current_tab:     'd',
            new_tab:                't',
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
pub struct Format {
    pub margin:      u8,
    pub list_bullet: String,
}
impl Format {
    pub fn default() -> Self {
        Self {
            margin:      2,
            list_bullet: "- ".into(),
        }
    }
}
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
impl Colors {
    pub fn default() -> Self {
        Self {
            background: (205, 205, 205),
            ui:         (205, 205, 205),
            text:       (205, 205, 205),
            heading1:   (205, 205, 205),
            heading2:   (205, 205, 205),
            heading3:   (205, 205, 205),
            link:       (205, 205, 205),
            badlink:    (205, 205, 205),
            quote:      (205, 205, 205),
            listitem:   (205, 205, 205),
            preformat:  (205, 205, 205),
        }
    }
    pub fn get_background(&self) -> Color {
        Color::Rgb {
            r: self.background.0,
            g: self.background.1,
            b: self.background.2,
        }
    }
    pub fn from_gem_doc(&self, doc: &GemDoc) -> Vec<ColoredText> {
        doc.doc.iter()
            .map(|(gem_type, text)| self.from_gem_type(gem_type, &text))
            .collect()
    }
    pub fn from_gem_type(&self, gem: &GemType, text: &str) -> ColoredText {
        let color = match gem {
            GemType::HeadingOne => Color::Rgb {
                    r: self.heading1.0, 
                    g: self.heading1.1, 
                    b: self.heading1.2},
            GemType::HeadingTwo => Color::Rgb {
                    r: self.heading2.0, 
                    g: self.heading2.1, 
                    b: self.heading2.2},
            GemType::HeadingThree => Color::Rgb {
                    r: self.heading3.0, 
                    g: self.heading3.1, 
                    b: self.heading3.2},
            GemType::Text => Color::Rgb {
                    r: self.text.0, 
                    g: self.text.1, 
                    b: self.text.2},
            GemType::Quote => Color::Rgb {
                    r: self.quote.0, 
                    g: self.quote.1, 
                    b: self.quote.2},
            GemType::ListItem => Color::Rgb {
                    r: self.listitem.0, 
                    g: self.listitem.1, 
                    b: self.listitem.2},
            GemType::PreFormat => Color::Rgb {
                    r: self.preformat.0, 
                    g: self.preformat.1, 
                    b: self.preformat.2},
            GemType::Link(_, _) => Color::Rgb {
                    r: self.link.0, 
                    g: self.link.1, 
                    b: self.link.2},
            GemType::BadLink(_) => Color::Rgb {
                    r: self.badlink.0, 
                    g: self.badlink.1, 
                    b: self.badlink.2},
        };
        ColoredText::new(text, color)
    }
}
