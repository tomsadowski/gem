// src/gem.rs

use crate::{
  util::{
    get_data, split_whitespace_once, Scheme, 
    join_if_relative,
  },
};
use url::{Url};

pub struct GemDoc {
  pub url:    Url,
  pub status: StatusText,
  pub doc:    Vec<GemText>,
}
impl GemDoc {
  pub fn new(url: &Url) -> Result<Self, String> {

    let (response, content) = get_data(url)
      .map_err(|e| e.to_string())?;

    let status = StatusText::parse(&response);

    let doc = match status.tag {
      Status::Success => 
        GemText::parse_doc(&content, url),

      _ => {
        let msg = format!(
          "response: status: {:?}, text: {}", 
          status.tag, 
          status.txt);
        vec![GemText::new(GemTag::Text, &msg)]
      }
    };

    let gem_doc = Self {
      url:    url.clone(),
      status: status,
      doc:    doc,
    };

    Ok(gem_doc)
  }
}

#[derive(Clone, PartialEq, Debug)]
pub struct GemText {
  pub tag: GemTag,
  pub txt: String,
}
impl GemText {
  pub fn new(tag: GemTag, txt: &str) -> Self {
    Self {
      tag, 
      txt: String::from(txt)
    }
  }

  pub fn parse_doc(text_str: &str, source: &Url) 
    -> Vec<Self> 
  {
    let mut vec = vec![];
    let mut preformat = false;

    for line in text_str.lines() {

      if let Some(("```", _)) = 
        line.split_at_checked(3)
      {
        preformat = !preformat;

      } else if preformat {
        vec.push(
          Self::new(
            GemTag::PreFormat, 
            line.into(),
          ));

      } else {
        vec.push(Self::parse_formatted(line, source));
      }
    }
    vec
  }

  pub fn parse_formatted(line: &str, source: &Url) -> Self {
    // look for 3 character symbols
    if let Some(("###", text)) = 
      line.split_at_checked(3) 
    {
      return Self::new(GemTag::HeadingThree, text.into())
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = 
      line.split_at_checked(2) 
    {
      if symbol == "=>" {

        let (url_str, link_str) = 
          split_whitespace_once(text);

        match join_if_relative(source, url_str) {
          Ok(url) =>
            return Self::new(
              GemTag::Link(Scheme::from(&url), url), 
              link_str.into()),
          Err(s) => 
            return Self::new(
              GemTag::BadLink(
                s.to_string()), 
                link_str.into())
        }

      } else if symbol == "##" {
        return Self::new(GemTag::HeadingTwo, text.into())
      }
    }

    // look for 1 character symbols
    if let Some((symbol, text)) = 
      line.split_at_checked(1) 
    {
      if symbol == ">" {
        return Self::new(GemTag::Quote, text.into())

      } else if symbol == "*" {
        return Self::new(GemTag::ListItem, 
                &format!("- {}", text))

      } else if symbol == "#" {
        return Self::new(GemTag::HeadingOne, text.into())
      }
    }
    return Self::new(GemTag::Text, line.into())
  }
}

#[derive(Clone, PartialEq, Debug)]
pub enum GemTag {
  HeadingOne,
  HeadingTwo,
  HeadingThree,
  Text, 
  PreFormat,
  Link(Scheme, Url),
  BadLink(String),
  ListItem,
  Quote,
} 

#[derive(Debug, Clone)]
pub struct StatusText {
  pub tag: Status, 
  pub txt: String,
}
impl StatusText {
  pub fn parse(line: &str) -> Self {

    let (code_str, msg) = split_whitespace_once(line);
    let status = Status::from(code_str);

    Self {
      tag: status,
      txt: msg.into()
    }
  }
}

#[derive(Debug, Clone)]
pub enum Status {
  InputExpected,
  InputExpectedSensitive,
  Success,
  RedirectTemporary,
  RedirectPermanent,
  FailTemporary,
  FailServerUnavailable,
  FailCGIError,
  FailProxyError,
  FailSlowDown,
  FailPermanent,
  FailNotFound,             
  FailGone,                 
  FailProxyRequestRefused,  
  FailBadRequest,           
  CertRequiredClient,
  CertRequiredTransient,   
  CertRequiredAuthorized,  
  CertNotAccepted,         
  FutureCertRejected,      
  ExpiredCertRejected,     
  Unknown(u8),
  Junk(String),
}
impl From<&str> for Status {
  fn from(item: &str) -> Status {
    match item.parse::<u8>()
      .map_err(|e| e.to_string()) 
    {
      Ok(u) => match u {
        10 | 12..=19 => Status::InputExpected,
        11 =>           Status::InputExpectedSensitive,
        20..=29 =>      Status::Success,
        30 | 32..=39 => Status::RedirectTemporary,
        31 =>           Status::RedirectPermanent,
        41 =>           Status::FailServerUnavailable,
        40 | 45..=49 => Status::FailTemporary,
        42 =>           Status::FailCGIError,
        43 =>           Status::FailProxyError,
        44 =>           Status::FailSlowDown,
        50 | 54..=58 => Status::FailPermanent,
        51 =>           Status::FailNotFound,
        52 =>           Status::FailGone,
        53 =>           Status::FailProxyRequestRefused,
        59 =>           Status::FailBadRequest,
        60 | 66..=69 => Status::CertRequiredClient,
        61 =>           Status::CertRequiredTransient,
        62 =>           Status::CertRequiredAuthorized,
        63 =>           Status::CertNotAccepted,
        64 =>           Status::FutureCertRejected,
        65 =>           Status::ExpiredCertRejected,
        u =>            Status::Unknown(u),
      } 

      Err(e) => Status::Junk(e)
    }
  }
}
