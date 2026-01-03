// config

use serde::Deserialize;

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
    pub move_page_up: char,
    pub move_page_down: char,
    pub cycle_to_left_tab: char,
    pub cycle_to_right_tab: char,
    pub inspect_under_cursor: char,
    pub delete_current_tab: char,
    pub new_tab: char,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Format {
    pub margin: u8,
    pub listbullet: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub init_url: String,
    pub colors: Colors,
    pub keys: Keys,
    pub format: Format,
}
impl Config {
    pub fn new(text: &str) -> Self {
        toml::from_str(text).unwrap()
    }
}
