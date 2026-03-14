// src/usr_layout.rs

use crate::{
  gem::{GemTag, GemText},
  text::{Text},
  screen::{Rect, Frame},
  util::{parse_color},
  usr_text::UserText,
};
use crossterm::style::Color;
use toml::{Table, Value};

#[derive(Debug)]
enum Key {
  Color(ColorKey), 
  Text(TextKey), 
  U16(U16Key),
}
#[derive(Debug)]
enum ColorKey {
  Bg, Banner, Border, Dlg,
}
#[derive(Debug)]
enum TextKey {
  Text, H1, H2, H3, Link, BadLink, Quote, List, Preformat,
}
#[derive(Debug)]
enum U16Key {
  XPage, YPage, XText, YText, ScrollAt,
}

impl Key {
  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "x_text"            => Ok(Self::U16(U16Key::XText)),
      "y_text"            => Ok(Self::U16(U16Key::YText)),
      "x_page"            => Ok(Self::U16(U16Key::XPage)),
      "y_page"            => Ok(Self::U16(U16Key::YPage)),
      "scroll_at"         => Ok(Self::U16(U16Key::ScrollAt)),
      "background" | "bg" => Ok(Self::Color(ColorKey::Bg)),
      "dialog" | "dlg"    => Ok(Self::Color(ColorKey::Dlg)),
      "banner"            => Ok(Self::Color(ColorKey::Banner)),
      "border"            => Ok(Self::Color(ColorKey::Border)),
      "text"              => Ok(Self::Text(TextKey::Text)),
      "header1" | "h1"    => Ok(Self::Text(TextKey::H1)),
      "header2" | "h2"    => Ok(Self::Text(TextKey::H2)),
      "header3" | "h3"    => Ok(Self::Text(TextKey::H3)),
      "link"              => Ok(Self::Text(TextKey::Link)),
      "badlink"           => Ok(Self::Text(TextKey::BadLink)),
      "quote"             => Ok(Self::Text(TextKey::Quote)),
      "list"              => Ok(Self::Text(TextKey::List)),
      "preformat"         => Ok(Self::Text(TextKey::Preformat)),
      key => 
        Err(format!("Layout table does not contain key {}.", key)),
    }
  }
}
impl ColorKey {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<Color, String>
  {
    parse_color(value)
      .map_err(|e| format!("{:?} : {}", self, e))
  }
}
impl TextKey {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<UserText, String>
  {
    if let Value::Table(t) = value {
      UserText::default()
        .read_table(t)
        .map_err(|e| format!("{:?} : {}", self, e))
    } else {
      Err(format!("prefix doesnt take {:?}", value))
    }
  }
}
impl U16Key {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<u16, String>
  {
    if let Value::Integer(t) = value {
        u16::try_from(*t)
        .map_err(|e| format!("{:?} : {}", self, e))
    } else {
      Err(format!("prefix doesnt take {:?}", value))
    }
  }
}

#[derive(Clone)]
pub struct UserLayout {
  pub x_text:    u16,
  pub y_text:    u16,
  pub x_page:    u16,
  pub y_page:    u16,
  pub scroll_at: u16,
  pub background: Option<Color>,
  pub banner:     Option<Color>,
  pub border:     Option<Color>,
  pub dialog:     Option<Color>,
  pub text:       UserText,
  pub heading1:   UserText,
  pub heading2:   UserText,
  pub heading3:   UserText,
  pub link:       UserText,
  pub badlink:    UserText,
  pub quote:      UserText,
  pub list:       UserText,
  pub preformat:  UserText,
} 

impl Default for UserLayout {
  fn default() -> Self {
    Self {
      scroll_at:  3,
      x_text:     0,
      y_text:     0,
      x_page:     0,
      y_page:     0,
      background: None,
      banner:     None,
      border:     None,
      dialog:     None,
      text:       UserText::default(),
      heading1:   UserText::default(),
      heading2:   UserText::default(),
      heading3:   UserText::default(),
      link:       UserText::default(),
      badlink:    UserText::default(),
      quote:      UserText::default(),
      list:       UserText::default(),
      preformat:  UserText::default(),
    }
  }
}

impl UserLayout {

  fn try_assign(&mut self, key: &Key, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      Key::Color(key) => {
        let v = key.try_parse_value(&value)?;
        match key {
          ColorKey::Bg     => self.background = Some(v),
          ColorKey::Banner => self.banner     = Some(v),
          ColorKey::Border => self.border     = Some(v),
          ColorKey::Dlg    => self.dialog     = Some(v),
        }
      }
      Key::U16(key) => {
        let v = key.try_parse_value(&value)?;
        match key {
          U16Key::XText    => self.x_text    = v,
          U16Key::YText    => self.y_text    = v,
          U16Key::XPage    => self.x_page    = v,
          U16Key::YPage    => self.y_page    = v,
          U16Key::ScrollAt => self.scroll_at = v,
        }
      }
      Key::Text(key) => {
        let v = key.try_parse_value(&value)?;
        match key {
          TextKey::Text      => self.text       = v,
          TextKey::H1        => self.heading1   = v,
          TextKey::H2        => self.heading2   = v,
          TextKey::H3        => self.heading3   = v,
          TextKey::Link      => self.link       = v,
          TextKey::BadLink   => self.badlink    = v,
          TextKey::Quote     => self.quote      = v,
          TextKey::List      => self.list       = v,
          TextKey::Preformat => self.preformat  = v,
        }
      }
    }
    Ok(())
  }

  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = Key::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }

    Ok(self)
  }

  pub fn get_rect_from_dim(&self, w: u16, h: u16) -> Rect {
    Rect::new(w, h)
      .crop_x(self.x_page)
      .crop_y(self.y_page)
  }

  pub fn get_frame_from_rect(&self, rect: &Rect) -> Frame {
    Frame {
      inner: rect
        .crop_x(self.scroll_at)
        .crop_y(self.scroll_at),
      outer: rect.clone(),
    }
  }

  pub fn gemtext_to_text(&self, gem: &Vec<GemText>) 
    -> Vec<Text>
  {
    gem.iter()
      .map(|gem| self.get_user_text(&gem)).collect()
  }

  pub fn get_user_text(&self, gtxt: &GemText) -> Text {
    match gtxt.tag {
      GemTag::HeadingOne => 
        self.heading1.get_text(&gtxt.txt).wrap(),

      GemTag::HeadingTwo => 
        self.heading2.get_text(&gtxt.txt).wrap(),

      GemTag::HeadingThree => 
        self.heading3.get_text(&gtxt.txt).wrap(),

      GemTag::Text => 
        self.text.get_text(&gtxt.txt).wrap(),

      GemTag::PreFormat => 
        self.preformat.get_text(&gtxt.txt),

      GemTag::Link(_, _) => 
        self.link.get_text(&gtxt.txt).wrap(),

      GemTag::BadLink(_) => 
        self.badlink.get_text(&gtxt.txt),

      GemTag::ListItem => 
        self.list.get_text(&gtxt.txt).wrap(),

      GemTag::Quote => 
        self.quote.get_text(&gtxt.txt).wrap(),
    }
  }
}
