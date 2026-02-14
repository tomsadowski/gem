// src/gem.rs

use crate::util::{
    get_data, split_whitespace_once, Scheme, parse_scheme, join_if_relative,
};
use url::{Url};

pub struct GemDoc {
    pub url:    Url,
    pub status: Status,
    pub msg:    String,
    pub doc:    Vec<(GemType, String)>,
}
impl GemDoc {
    pub fn new(url: &Url) -> Result<Self, String> {
        let (response, content) = get_data(url)
            .map_err(|e| e.to_string())?;
        let (status, msg) = parse_status(&response)
            .map_err(|e| e.to_string())?;
        let doc = match status {
            Status::Success => parse_doc(&content, url),
            _ => {
                let msg = 
                    format!("rspns: stts: {:?}, msg: {}", status, msg);
                vec![(GemType::Text, msg)]
            }
        };
        let gem_doc = Self {
            url:    url.clone(),
            status: status,
            msg:    msg,
            doc:    doc,
        };
        Ok(gem_doc)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum GemType {
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
}

pub fn parse_doc(text_str: &str, source: &Url) -> Vec<(GemType, String)> {
    let mut vec = vec![];
    let mut preformat = false;
    for line in text_str.lines() {
        if let Some(("```", _)) = line.split_at_checked(3) {
            preformat = !preformat;
        } else if preformat {
            vec.push((GemType::PreFormat, line.into()));
        } else {
            let (gem, text) = parse_formatted(line, source);
            vec.push((gem, text.into()));
        }
    }
    vec
}

fn parse_formatted(line: &str, source: &Url) -> (GemType, String) {
    // look for 3 character symbols
    if let Some(("###", text)) = line.split_at_checked(3) {
        return (GemType::HeadingThree, text.into())
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = line.split_at_checked(2) {
        if symbol == "=>" {
            let (url_str, link_str) = split_whitespace_once(text);
            match join_if_relative(source, url_str) {
                Ok(url) =>
                    return (
                        GemType::Link(parse_scheme(&url), url), 
                        link_str.into()),
                Err(s) => 
                    return (GemType::BadLink(s.to_string()), link_str.into())
            }
        } else if symbol == "##" {
            return (GemType::HeadingTwo, text.into())
        }
    }
    // look for 1 character symbols
    if let Some((symbol, text)) = line.split_at_checked(1) {
        if symbol == ">" {
            return (GemType::Quote, text.into())
        } else if symbol == "*" {
            return (GemType::ListItem, format!("- {}", text))
        } else if symbol == "#" {
            return (GemType::HeadingOne, text.into())
        }
    }
    return (GemType::Text, line.into())
}

pub fn parse_status(line: &str) -> Result<(Status, String), String> {
    let (code_str, msg) = split_whitespace_once(line);
    let code = code_str.parse::<u8>().map_err(|e| e.to_string())?;
    let status = get_status(code)?;
    Ok((status, msg.into()))
}

fn get_status(code: u8) -> Result<Status, String> {
    match code {
        10 | 12..=19 => Ok(Status::InputExpected),
        11 =>           Ok(Status::InputExpectedSensitive),
        20..=29 =>      Ok(Status::Success),
        30 | 32..=39 => Ok(Status::RedirectTemporary),
        31 =>           Ok(Status::RedirectPermanent),
        41 =>           Ok(Status::FailServerUnavailable),
        40 | 45..=49 => Ok(Status::FailTemporary),
        42 =>           Ok(Status::FailCGIError),
        43 =>           Ok(Status::FailProxyError),
        44 =>           Ok(Status::FailSlowDown),
        50 | 54..=58 => Ok(Status::FailPermanent),
        51 =>           Ok(Status::FailNotFound),
        52 =>           Ok(Status::FailGone),
        53 =>           Ok(Status::FailProxyRequestRefused),
        59 =>           Ok(Status::FailBadRequest),
        60 | 66..=69 => Ok(Status::CertRequiredClient),
        61 =>           Ok(Status::CertRequiredTransient),
        62 =>           Ok(Status::CertRequiredAuthorized),
        63 =>           Ok(Status::CertNotAccepted),
        64 =>           Ok(Status::FutureCertRejected),
        65 =>           Ok(Status::ExpiredCertRejected),
        _ =>            Err(format!("invalid status number: {}", code)),
    }
}
