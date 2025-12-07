// dialog

use crate::{
    gemini::{
        self, Scheme, GemTextData,
    }, 
    util,
};
use std::io::{
    self, Write, Stdout
};
use crossterm::{
    QueueableCommand, cursor, terminal,
    style::{
        self, Color, Colors,
    },
};
use url::Url;

#[derive(Clone, Debug)]
pub struct LineStyles {
    pub heading_one:   Colors,
    pub heading_two:   Colors,
    pub heading_three: Colors,
    pub link:          Colors,
    pub list_item:     Colors,
    pub quote:         Colors,
    pub preformat:     Colors,
    pub text:          Colors,
    pub plaintext:     Colors,
}
impl LineStyles {
    pub fn new() -> Self {
        let heading_one_style = 
            Colors::new(
                Color::Rgb {r: 208,  g:  96,  b:  96},
                Color::Rgb {r:  48,  g:  24,  b:  24},
            );
        let heading_two_style = 
            Colors::new(
                Color::Rgb {r: 208,  g:  96,  b:  96},
                Color::Rgb {r:   0,  g:   0,  b:   0},
            );
        let heading_three_style = 
            Colors::new(
                Color::Rgb {r: 208,  g:  96,  b:  96},
                Color::Rgb {r:   0,  g:   0,  b:   0},
            );
        let link_style = 
            Colors::new(
                Color::Rgb {r: 176,  g:  96,  b: 192},
                Color::Rgb {r:   0,  g:   0,  b:   0},
            );
        let text_style =
            Colors::new(
                Color::Rgb {r: 192,  g: 192,  b: 144},
                Color::Rgb {r:   0,  g:   0,  b:   0},
            );
        let list_style =
            Colors::new(
                Color::Rgb {r: 192,  g: 192,  b: 144},
                Color::Rgb {r:   0,  g:   0,  b:   0},
            );
        let quote_style =
            Colors::new(
                Color::Rgb {r: 192,  g: 192,  b: 144},
                Color::Rgb {r:   0,  g:   0,  b:   0},
            );
        Self {
            heading_one:   heading_one_style,
            heading_two:   heading_two_style,
            heading_three: heading_three_style,
            link:          link_style,
            list_item:     list_style,
            quote:         quote_style,
            preformat:     text_style,
            plaintext:     text_style,
            text:          text_style,
        }
    }
    pub fn get_colors(&self, data: gemini::GemTextData) -> Colors {
        match data {
            gemini::GemTextData::HeadingOne   => self.heading_one,
            gemini::GemTextData::HeadingTwo   => self.heading_two,
            gemini::GemTextData::HeadingThree => self.heading_three,
            gemini::GemTextData::Text         => self.text,
            gemini::GemTextData::Quote        => self.quote,
            gemini::GemTextData::ListItem     => self.list_item,
            gemini::GemTextData::PreFormat    => self.preformat,
            _ => self.link,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    FollowLink(Url),
    Download,
    Acknowledge,
}
#[derive(Clone, Debug)]
pub struct Dialog {
    pub action: Action,
    pub text:   String,
}
impl Dialog {
    // Dialog asking to download resource
    pub fn download(str: String) -> Self {
        Self { 
            action: Action::Download, 
            text:   format!("Download nontext type: {}?", str)
        }
    }

    // Dialog asking for acknowledgement 
    pub fn acknowledge(str: String) -> Self {
        Self { 
            action: Action::Acknowledge, 
            text:   format!("{}?", str)
        }
    }

    // Dialog asking to go to new url
    pub fn follow_link(url: Url) -> Self {
        Self { 
            action: Action::FollowLink(url.clone()), 
            text:   format!("Go to {}?", String::from(url))
        }
    }

    pub fn query_gemtext_data(text: GemTextData) -> Option<Dialog> {
        match text {
            GemTextData::Link(Scheme::Gemini(url)) => {
                Some(Dialog::follow_link(url))
            }
            g => {
                Some(Dialog::acknowledge(format!("{:?}", g)))
            }
        }
    }
}
