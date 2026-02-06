// config

use crate::{
    gemini::{GemType, GemDoc, Scheme},
    text::{ColoredText},
};
use crossterm::{
    style::{self, Color},
    event::{Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
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
        let result = toml::from_str(text);
        match result {
            Ok(cfg) => cfg,
            _ => Self::default(),
        }
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
#[derive(Deserialize, Debug, Clone)]
pub struct KeyParams {
    pub global:         char,
    pub load_cfg:       char,
    pub msg_view:       char,
    pub tab_view:       char,
    pub dialog: DialogKeyParams,
    pub tab:    TabKeyParams,
}
impl KeyParams {
    pub fn default() -> Self {
        Self {
            global:         'g',
            load_cfg:       'c',
            msg_view:       'm',
            tab_view:       't',
            dialog: DialogKeyParams::default(),
            tab:    TabKeyParams::default(),
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Deserialize, Debug, Clone)]
pub struct DialogKeyParams {
    pub ack: char,
    pub yes: char,
    pub no:  char,
}
impl DialogKeyParams {
    pub fn default() -> Self {
        Self {
            ack: 'y',
            yes: 'y',
            no:  'n',
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
pub struct FormatParams {
    pub margin: u16,
    // gemini
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
#[derive(Deserialize, Debug, Clone)]
pub struct ColorParams {
    pub background: (u8, u8, u8),
    pub banner:     (u8, u8, u8),
    pub dialog:     (u8, u8, u8),
    // gemini
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
        Color::Rgb {
            r: self.banner.0,
            g: self.banner.1,
            b: self.banner.2,
        }
    }
    pub fn get_dialog(&self) -> Color {
        Color::Rgb {
            r: self.dialog.0,
            g: self.dialog.1,
            b: self.dialog.2,
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
                    r: self.list.0, 
                    g: self.list.1, 
                    b: self.list.2},
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
