// gem/src/config

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GemColors {
    pub heading1:  (char, char, char),
    pub heading2:  (char, char, char),
    pub heading3:  (char, char, char),
    pub text:      (char, char, char),
    pub link:      (char, char, char),
    pub quote:     (char, char, char),
    pub listitem:  (char, char, char),
    pub preformat: (char, char, char),
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
pub struct Config {
    pub init_url: String,
    pub gemcolors: GemColors,
    pub keys: Keys,
}
impl Config {
    pub fn new(text: &str) -> Self {
        toml::from_str(text).unwrap()
    }
}
