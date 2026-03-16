// src/usr.rs

use crate::{
  gem::{GemDoc, GemTag, GemText},
  text::{Doc, Text, Editor},
  page::{Rect, Page},
  util::{parse_color},
  dlg::{Dialog, InputType},
};
use crossterm::{
  style::Color,
  event::KeyCode,
};
use toml::{Table, Value};

// module: usr
//
// a)   Parse '.gemset' file.
//
// b)   Help construct items
//      that require many user parameters.
//
// (a) read file, (b) co-author runtime data.


#[derive(Debug)]
enum UserKey {
  InitUrl,
  Layout,
  Keys,
}
impl UserKey {

  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "init_url" => Ok(Self::InitUrl),
      "layout"   => Ok(Self::Layout),
      "keys"     => Ok(Self::Keys),
      key => 
        Err(
          format!(
            "No key named {} in User table", key)),
    }
  }
}


#[derive(Clone)]
pub struct User {
  pub init_url:  String,
  pub layout:    UserLayout,
  pub keys:      UserKeys,
} 
impl Default for User {

  fn default() -> Self {
    Self {
      init_url: "gemini://datapulp.smol.pub/".into(),
      layout:    UserLayout::default(),
      keys:      UserKeys::default(),
    }
  }
}
impl User {

  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = UserKey::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }
    Ok(self)
  }


  pub fn parse(text: &str) -> Result<Self, String> {
    let table = text.parse::<Table>()
      .map_err(|e| e.to_string())?;
    Self::default().read_table(&table)
  }


  fn try_assign(&mut self, key: &UserKey, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      UserKey::InitUrl => {

        if let Value::String(s) = value {
          self.init_url = s.into();

        } else {
          return Err(
            "init_url key expects a string value".into())
        }
      }
      UserKey::Layout => {

        if let Value::Table(t) = value {
          self.layout = UserLayout::default()
            .read_table(t)?;

        } else {
          return Err(
            "layout key expects a table value".into())
        }
      }
      UserKey::Keys => {

        if let Value::Table(t) = value {
          self.keys = UserKeys::default()
            .read_table(t)?;

        } else {
          return Err(
            "keys key expects a table value".into())
        }
      }
    }
    Ok(())
  }


  pub fn get_layout(&self, w: u16, h: u16) -> (Page, Page) 
  {
    let rect = Rect::new(w, h);
    self.layout.get_layout(&rect)
  }


  pub fn get_hdr_doc(&self, info: &str, page: &Page) 
    -> Doc 
  {
    let fg = self.layout.banner
      .unwrap_or(Color::White);
    let bg = self.layout.background
      .unwrap_or(Color::Black);
    let line = &String::from("-")
      .repeat(page.text.w);

    Doc::new(
      vec![
        Text::from(info).fg(fg).bg(bg),
        Text::from(line.as_str()).fg(fg).bg(bg), 
      ],
      &page
    )
  }


  pub fn text(&self, page: &Page, text: &str) -> Dialog {

    let mut dlg = Dialog::new(page, text);
    let pos = dlg.input_page.pos();
    let color = self.layout.dialog.unwrap_or(Color::White);
    let editor = Editor::new(&dlg.input_page, "", color);

    dlg.input_type = InputType::Text(editor, pos);
    dlg
  }


  pub fn ack(&self, page: &Page, text: &str) -> Dialog {

    let mut dlg = Dialog::new(page, text);
    dlg.input_type = InputType::Ack(self.keys.ack);
    dlg
  }


  pub fn ask(&self, page: &Page, text: &str) -> Dialog {

    let mut dlg = Dialog::new(page, text);
    dlg.input_type = InputType::Ask
      (self.keys.yes, self.keys.no);
    dlg
  }


  pub fn get_doc(&self, gdoc: &GemDoc, page: &Page) -> Doc {
    let text = self.layout.gemtext_to_text(&gdoc.doc);
    Doc::new(text, &page)
  }


  pub fn get_page(&self, w: u16, h: u16) -> Page {
    let rect = Rect::new(w, h);
    self.layout.get_page_from_rect(&rect)
  }
}


#[derive(Debug)]
enum KeysKey {
  Global, 
  MsgView, 
  LoadUser,
  TabView, 
  MoveUp, 
  MoveDown, 
  MoveLeft, 
  MoveRight,
  CycleLeft, 
  CycleRight, 
  DelTab, 
  NewTab, 
  Inspect, 
  Ack, 
  Yes, 
  No, 
  Cancel,
}
impl KeysKey {

  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "global"      => Ok(Self::Global),
      "msg_view"    => Ok(Self::MsgView),
      "tab_view"    => Ok(Self::TabView),
      "load_usr"    => Ok(Self::LoadUser),
      "move_up"     => Ok(Self::MoveUp),
      "move_down"   => Ok(Self::MoveDown),
      "move_left"   => Ok(Self::MoveLeft),
      "move_right"  => Ok(Self::MoveRight),
      "cycle_left"  => Ok(Self::CycleLeft),
      "cycle_right" => Ok(Self::CycleRight),
      "delete_tab"  => Ok(Self::DelTab),
      "new_tab"     => Ok(Self::NewTab),
      "inspect"     => Ok(Self::Inspect),
      "ack"         => Ok(Self::Ack),
      "yes"         => Ok(Self::Yes),
      "no"          => Ok(Self::No),
      "cancel"      => Ok(Self::Cancel),
      key => 
        Err(
          format!(
            "KeysKeys table does not contain key {}.", key)),
    }
  }
}


#[derive(Clone)]
pub struct UserKeys {
  pub global:      KeyCode,
  pub cancel:      KeyCode,
  pub load_usr:    KeyCode,
  pub msg_view:    KeyCode,
  pub tab_view:    KeyCode,
  pub move_up:     KeyCode,
  pub move_down:   KeyCode,
  pub move_left:   KeyCode,
  pub move_right:  KeyCode,
  pub cycle_left:  KeyCode,
  pub cycle_right: KeyCode,
  pub inspect:     KeyCode,
  pub delete_tab:  KeyCode,
  pub new_tab:     KeyCode,
  pub ack:         KeyCode, 
  pub yes:         KeyCode, 
  pub no:          KeyCode
} 
impl Default for UserKeys {

  fn default() -> Self {
    Self {
      global:      KeyCode::Char('g'),
      cancel:      KeyCode::Esc,
      load_usr:    KeyCode::Char('c'),
      msg_view:    KeyCode::Char('m'),
      tab_view:    KeyCode::Char('t'),
      move_up:     KeyCode::Up,
      move_down:   KeyCode::Down,
      move_left:   KeyCode::Left,
      move_right:  KeyCode::Right,
      cycle_left:  KeyCode::Char('E'),
      cycle_right: KeyCode::Char('N'),
      inspect:     KeyCode::Enter,
      delete_tab:  KeyCode::Char('d'),
      new_tab:     KeyCode::Char('n'),
      ack:         KeyCode::Enter, 
      yes:         KeyCode::Char('y'), 
      no:          KeyCode::Char('n')
    }
  }
}
impl UserKeys {

  fn try_assign(&mut self, key: &KeysKey, value: &Value) 
    -> Result<(), String> 
  {
    let v = Self::try_from_value(value)?;
    match key {
      KeysKey::Global     => self.global = v,
      KeysKey::MsgView    => self.msg_view = v,
      KeysKey::LoadUser   => self.load_usr = v,
      KeysKey::TabView    => self.tab_view = v,
      KeysKey::MoveUp     => self.move_up = v,
      KeysKey::MoveDown   => self.move_down = v,
      KeysKey::MoveLeft   => self.move_left = v,
      KeysKey::MoveRight  => self.move_right = v,
      KeysKey::CycleLeft  => self.cycle_left = v,
      KeysKey::CycleRight => self.cycle_right = v,
      KeysKey::DelTab     => self.delete_tab = v,
      KeysKey::NewTab     => self.new_tab = v,
      KeysKey::Inspect    => self.inspect = v,
      KeysKey::Ack        => self.ack = v,
      KeysKey::Yes        => self.yes = v,
      KeysKey::No         => self.no = v,
      KeysKey::Cancel     => self.cancel = v,
    }
    Ok(())
  }


  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = KeysKey::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }
    Ok(self)
  }


  pub fn try_from_value(value: &Value) 
    -> Result<KeyCode, String> 
  {
    if let Value::String(s) = value {
      if let Some(kc) = Self::keycode_from_string(&s) {
        Ok(kc)
      } else {
        Err("i cant do it".into())
      }
    } else {
      Err("i cant do it".into())
    }
  }


  pub fn keycode_from_string(text: &str) -> Option<KeyCode> {
    match text {
      "esc" | "escape"  => Some(KeyCode::Esc),
      "ent" | "enter"   => Some(KeyCode::Enter),
      "space"           => Some(KeyCode::Char(' ')),
      "left"            => Some(KeyCode::Left),
      "up"              => Some(KeyCode::Up),
      "down"            => Some(KeyCode::Down),
      "right"           => Some(KeyCode::Right),
      t => 
        t
          .chars()
          .next()
          .map(|c| KeyCode::Char(c)),
    }
  }
}


#[derive(Debug)]
enum LayoutKey {
  Color(ColorLayoutKey), 
  Text(TextLayoutKey), 
  U16(U16LayoutKey),
}
impl LayoutKey {

  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "x_text" => 
        Ok(Self::U16(U16LayoutKey::XText)),

      "y_text" => 
        Ok(Self::U16(U16LayoutKey::YText)),

      "x_page" => 
        Ok(Self::U16(U16LayoutKey::XPage)),

      "y_page" => 
        Ok(Self::U16(U16LayoutKey::YPage)),

      "scroll_at" => 
        Ok(Self::U16(U16LayoutKey::ScrollAt)),

      "background" | "bg" => 
        Ok(Self::Color(ColorLayoutKey::Bg)),

      "dialog" | "dlg" => 
        Ok(Self::Color(ColorLayoutKey::Dlg)),

      "banner" => 
        Ok(Self::Color(ColorLayoutKey::Banner)),

      "border" => 
        Ok(Self::Color(ColorLayoutKey::Border)),

      "text" => 
        Ok(Self::Text(TextLayoutKey::Text)),

      "header1" | "h1" => 
        Ok(Self::Text(TextLayoutKey::H1)),

      "header2" | "h2" => 
        Ok(Self::Text(TextLayoutKey::H2)),

      "header3" | "h3" => 
        Ok(Self::Text(TextLayoutKey::H3)),

      "link" => 
        Ok(Self::Text(TextLayoutKey::Link)),

      "badlink" => 
        Ok(Self::Text(TextLayoutKey::BadLink)),

      "quote" => 
        Ok(Self::Text(TextLayoutKey::Quote)),

      "list" => 
        Ok(Self::Text(TextLayoutKey::List)),

      "preformat" => 
        Ok(Self::Text(TextLayoutKey::Preformat)),

      key => 
        Err(
          format!(
            "Layout table does not contain key {}.", key)),
    }
  }
}


#[derive(Debug)]
enum ColorLayoutKey {
  Bg, Banner, Border, Dlg,
}
impl ColorLayoutKey {
  pub fn try_parse_value(&self, value: &Value) 
    -> Result<Color, String>
  {
    parse_color(value)
      .map_err(|e| format!("{:?} : {}", self, e))
  }
}


#[derive(Debug)]
enum TextLayoutKey {
  Text, 
  H1, 
  H2, 
  H3, 
  Link, 
  BadLink, 
  Quote, 
  List, 
  Preformat,
}
impl TextLayoutKey {
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


#[derive(Debug)]
enum U16LayoutKey {
  XPage, YPage, XText, YText, ScrollAt,
}
impl U16LayoutKey {
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

  fn try_assign(&mut self, key: &LayoutKey, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      LayoutKey::Color(key) => {
        let v = key.try_parse_value(&value)?;

        match key {
          ColorLayoutKey::Bg => 
            self.background = Some(v),

          ColorLayoutKey::Banner => 
            self.banner = Some(v),

          ColorLayoutKey::Border => 
            self.border = Some(v),

          ColorLayoutKey::Dlg => 
            self.dialog = Some(v),
        }
      }
      LayoutKey::U16(key) => {
        let v = key.try_parse_value(&value)?;
        match key {
          U16LayoutKey::XText => 
            self.x_text = v,

          U16LayoutKey::YText => 
            self.y_text = v,

          U16LayoutKey::XPage => 
            self.x_page = v,

          U16LayoutKey::YPage => 
            self.y_page = v,

          U16LayoutKey::ScrollAt => 
            self.scroll_at = v,
        }
      }
      LayoutKey::Text(key) => {
        let v = key.try_parse_value(&value)?;
        match key {
          TextLayoutKey::Text => 
            self.text = v,

          TextLayoutKey::H1 => 
            self.heading1 = v,

          TextLayoutKey::H2 => 
            self.heading2 = v,

          TextLayoutKey::H3 => 
            self.heading3 = v,

          TextLayoutKey::Link => 
            self.link = v,

          TextLayoutKey::BadLink => 
            self.badlink = v,

          TextLayoutKey::Quote => 
            self.quote = v,

          TextLayoutKey::List => 
            self.list = v,

          TextLayoutKey::Preformat => 
            self.preformat = v,
        }
      }
    }
    Ok(())
  }


  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = LayoutKey::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }
    Ok(self)
  }


  // called infrequently, construct many things
  // based on screensize and usr
  pub fn get_layout(&self, rect: &Rect) -> (Page, Page) {

    let (hdr_rect, tab_rect) = {
      let rect = rect
          .crop_x(self.x_page)
          .crop_y(self.y_page);
      (rect.crop_south(rect.y().len16() - 2), 
       rect.crop_north(2))
    };
    let hdr = Page::new(&hdr_rect)
      .text(self.x_text, 0);
    let tab = Page::new(&tab_rect)
      .text(self.x_text, self.y_text)
      .scroll(self.scroll_at, self.scroll_at);
    (hdr, tab)
  }


  pub fn get_rect_from_dim(&self, w: u16, h: u16) -> Rect {
    Rect::new(w, h)
      .crop_x(self.x_page)
      .crop_y(self.y_page)
  }


  pub fn get_page_from_rect(&self, rect: &Rect) -> Page {
    let rect = rect
      .crop_x(self.x_page)
      .crop_y(self.y_page);
    Page::new(&rect)
      .text(self.x_text, self.y_text)
      .scroll(self.scroll_at, self.scroll_at)
  }


  pub fn gemtext_to_text(&self, gem: &Vec<GemText>) 
    -> Vec<Text>
  {
    gem.iter()
      .map(|gem| self.get_user_text(&gem)).collect()
  }


  pub fn get_user_text(&self, gtxt: &GemText) -> Text {
    let text = match gtxt.tag {
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
    };
    text.bg(self.background.unwrap_or(Color::Black))
  }
}


#[derive(Debug)]
enum TextKey {
  Color(ColorTextKey), 
  Usize(UsizeTextKey), 
  Prefix,
}
impl TextKey {

  pub fn try_from_string(key: &str) 
    -> Result<Self, String> 
  {
    match key {
      "fg" => 
        Ok(Self::Color(ColorTextKey::Fg)),

      "bg" => 
        Ok(Self::Color(ColorTextKey::Bg)),

      "above" => 
        Ok(Self::Usize(UsizeTextKey::Above)),

      "below" => 
        Ok(Self::Usize(UsizeTextKey::Below)),

      "prefix" => 
        Ok(Self::Prefix),

      key => 
        Err(
          format!(
            "{} no such field in the table", key)),
    }
  }
}


#[derive(Debug)]
enum ColorTextKey {
  Fg, Bg,
}
impl ColorTextKey {

  pub fn try_parse_value(&self, value: &Value) 
    -> Result<Color, String>
  {
    parse_color(value)
      .map_err(|e| format!("{:?} : {}", self, e))
  }
}


#[derive(Debug)]
enum UsizeTextKey {
  Above, Below,
}
impl UsizeTextKey {

  pub fn try_parse_value(&self, value: &Value) 
    -> Result<usize, String>
  {
    match value {
      Value::Integer(i) => 
        usize::try_from(*i)
          .map_err(|e| format!("{:?} : {}", self, e)),
      v => 
        Err(format!("{:?} doesn't take {:?}", self, v)),
    }
  }
}


#[derive(Clone)]
pub struct UserText {
  pub fg: Option<Color>,
  pub bg: Option<Color>,
  pub above: usize,
  pub below: usize,
  pub prefix: String,
} 
impl Default for UserText {

  fn default() -> Self {
    Self {
      fg: None,
      bg: None,
      above: 0,
      below: 0,
      prefix: "".to_string(),
    }
  }
}
impl UserText {

  pub fn get_text(&self, text: &str) -> Text {
    let mut text = Text::from(text)
      .above(self.above)
      .below(self.below)
      .prefix(&self.prefix);
    if let Some(fg) = self.fg {
      text = text.fg(fg);
    }
    if let Some(bg) = self.bg {
      text = text.bg(bg);
    }
    text
  }


  pub fn read_table(mut self, table: &Table) 
    -> Result<Self, String> 
  {
    for (key, value) in table.iter() {
      let k = TextKey::try_from_string(&key)?;
      self.try_assign(&k, value)?;
    }
    Ok(self)
  }


  fn try_assign(&mut self, key: &TextKey, value: &Value) 
    -> Result<(), String> 
  {
    match key {
      TextKey::Color(k) => {
        let v = k.try_parse_value(&value)?;
        match k {
          ColorTextKey::Fg => self.fg = Some(v),
          ColorTextKey::Bg => self.bg = Some(v),
        }
      }
      TextKey::Usize(k) => {
        let v = k.try_parse_value(&value)?;
        match k {
          UsizeTextKey::Above => self.above = v,
          UsizeTextKey::Below => self.below = v,
        }
      }
      TextKey::Prefix => {
        if let Value::String(s) = value {
          self.prefix = s.into(); 
        } else {
          return Err(
            format!("prefix doesnt take {:?}", value))
        }
      }
    }
    Ok(())
  }
}
