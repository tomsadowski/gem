// gemini

use url::Url;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseError;

pub const GOPHER_SCHEME: &str = "gopher";
pub const HTTPS_SCHEME: &str  = "https";
pub const HTTP_SCHEME: &str   = "http";

pub const INIT_LINK: &str     = "gemini://geminiprotocol.net/";
pub const STATUS_REGEX: &str  = r"^(\d{1,3})[ \t](.*)\r\n$";
pub const LINK_REGEX: &str    = r"^\s*(\S*)\s*(.*)?$";

pub const LINK_SYMBOL: &str      = "=>";
pub const TOGGLE_SYMBOL: &str    = "```";
pub const QUOTE_SYMBOL: &str     = ">";
pub const LIST_ITEM_SYMBOL: &str = "*";
pub const HEADING_1_SYMBOL: &str   = "#";
pub const HEADING_2_SYMBOL: &str   = "##";
pub const HEADING_3_SYMBOL: &str   = "###";

pub const GEMINI_PORT: &str   = "1965";
pub const GEMINI_SCHEME: &str = "gemini";

#[derive(Clone, Debug)]
pub struct GeminiResponse {
    pub header: Status,
    pub data: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct GemTextDoc {
    pub text: Vec<GemTextLine>,
} 
impl GemTextDoc {
    pub fn new(data: String) -> Self {
        let lines = data
            .lines()
            .map(|l| GemTextLine::from_str(l).unwrap())
            .collect();
        Self {
            text: lines
        }
    }
}
#[derive(Clone, Debug)]
pub enum Heading { 
    One(String), 
    Two(String),
    Three(String),
}
#[derive(Clone, Debug)]
pub enum GemTextLine {
    Text(String), 
    Link(Link),
    Heading(Heading),
    ListItem(String),
    Quote(String),
    Toggle,
} 
impl FromStr for GemTextLine {
    type Err = ParseError;
    fn from_str(line: &str) -> Result<GemTextLine, Self::Err> {
        if let Some((symbol, text)) = line.split_at_checked(1) 
        {
            if symbol == QUOTE_SYMBOL 
            {
                return Ok(GemTextLine::Quote(text.to_string()))
            }
            if symbol == LIST_ITEM_SYMBOL 
            {
                return Ok(GemTextLine::ListItem(text.to_string()))
            }
            if symbol == HEADING_1_SYMBOL 
            {
                if let Some((symbol, text)) = line.split_at_checked(2) 
                {
                    if symbol == HEADING_2_SYMBOL 
                    {
                        if let Some((symbol, text)) = line.split_at_checked(3) 
                        {
                            if symbol == HEADING_3_SYMBOL 
                            {
                                return Ok(GemTextLine::Heading(Heading::Three(text.to_string())))
                            }
                        }
                        return Ok(GemTextLine::Heading(Heading::Two(text.to_string())))
                    }
                }
                return Ok(GemTextLine::Heading(Heading::One(text.to_string())))
            }
        }
        if let Some((symbol, text)) = line.split_at_checked(2) 
        {
            if symbol == LINK_SYMBOL 
            {
                let link = Link::from_str(text)?;
                return Ok(GemTextLine::Link(link))
            }
        }
        if let Some((symbol, text)) = line.split_at_checked(3) 
        {
            if symbol == TOGGLE_SYMBOL {
                return Ok(GemTextLine::Toggle)
            }
        }
        Ok(GemTextLine::Text(line.to_string()))
    }
}
#[derive(Clone, Debug)]
pub enum Link {
    Gemini(Url, String),
    Gopher(Url, String),
    Http(Url, String),
    Relative(String, String),
    Unknown(Url, String),
}
impl FromStr for Link {
    type Err = ParseError;
    fn from_str(line: &str) -> Result<Link, ParseError> {
        // get regex
        let Ok(regex) = Regex::new(LINK_REGEX)
            else {return Err(ParseError)};
        // get captures
        let Some(captures) = regex.captures(&line) 
            else {return Err(ParseError)};
        // get url
        let url_str = captures
            .get(1)
            .map_or("", |m| m.as_str())
            .to_string();
        let url_result = Url::parse(&url_str);
        // get label 
        let label_str = captures
            .get(2)
            .map_or("", |m| m.as_str());
        let label = if label_str.is_empty() {
            url_str.clone()
        } else {
            label_str.to_string()
        };
        // return Result
        if let Ok(url) = url_result {
            match url.scheme() {
                GEMINI_SCHEME => return Ok(Link::Gemini(url, label)),
                GOPHER_SCHEME => return Ok(Link::Gopher(url, label)),
                HTTP_SCHEME   => return Ok(Link::Http(url, label)),
                HTTPS_SCHEME  => return Ok(Link::Http(url, label)),
                _             => return Ok(Link::Unknown(url, label)),
            };
        } else if Err(url::ParseError::RelativeUrlWithoutBase) == url_result {
            Ok(Link::Relative(url_str, label))
        } else {
            Err(ParseError) 
        }
    }
}
#[derive(Debug, Clone)]
pub enum Status {
    Input(String),
    Secret(String),
    Success(String),
    SuccessEndOfClientCertificateSession(String),
    RedirectTemporary(String),
    RedirectPermanent(String),
    TemporaryFailure(String),
    ServerUnavailable(String),
    CGIError(String),
    ProxyError(String),
    SlowDown(String),
    PermanentFailure(String),
    NotFound(String),
    Gone(String),
    ProxyRequestRefused(String),
    BadRequest(String),
    ClientCertificateRequired(String),
    TransientCertificateRequired(String),
    AuthorisedCertificatedRequired(String),
    CertificateNotAccepted(String),
    FutureCertificateRejected(String),
    ExpiredCertificateRejected(String),
    Unknown(String),
} 
impl Status {
    // create status from integer
    pub fn new(code: i16, meta: String) -> Self {
        match code {
            10 => Status::Input(meta),
            11 => Status::Secret(meta),
            20 => Status::Success(meta),
            21 => Status::SuccessEndOfClientCertificateSession(meta),
            30 => Status::RedirectTemporary(meta),
            31 => Status::RedirectPermanent(meta),
            40 => Status::TemporaryFailure(meta),
            41 => Status::ServerUnavailable(meta),
            42 => Status::CGIError(meta),
            43 => Status::ProxyError(meta),
            44 => Status::SlowDown(meta),
            50 => Status::PermanentFailure(meta),
            51 => Status::NotFound(meta),
            52 => Status::Gone(meta),
            53 => Status::ProxyRequestRefused(meta),
            59 => Status::BadRequest(meta),
            60 => Status::ClientCertificateRequired(meta),
            61 => Status::TransientCertificateRequired(meta),
            62 => Status::AuthorisedCertificatedRequired(meta),
            63 => Status::CertificateNotAccepted(meta),
            64 => Status::FutureCertificateRejected(meta),
            65 => Status::ExpiredCertificateRejected(meta),
            _ => Status::Unknown(meta),
        }
    }
} 
impl FromStr for Status {
    type Err = ParseError;
    fn from_str(line: &str) -> Result<Status, Self::Err> {
        // get regex
        let Ok(regex) = Regex::new(STATUS_REGEX)
            else {return Err(ParseError)};
        // get captures
        let Some(captures) = regex.captures(&line) 
            else {return Err(ParseError)};
        // get code from captures
        let Ok(code) = captures
            .get(1)
            .map_or("", |m| m.as_str())
            .parse()
            else {return Err(ParseError)};
        // get meta from captures
        let meta = captures
            .get(2)
            .map_or("", |m| m.as_str())
            .to_string();
        // return Result
        Ok(Status::new(code, meta))
    }
}
