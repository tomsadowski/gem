// gemini

use crate::util;
use url::{
    Url, ParseError
};
use std::{
    time::{Duration}, 
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};
use native_tls::TlsConnector;

#[derive(Clone, PartialEq, Debug)]
pub enum Scheme {
    Gemini,
    Gopher,
    Http,
    Unknown,
}
pub fn parse_scheme(url: &Url) -> Scheme {
    match url.scheme() {
        "gemini" => Scheme::Gemini,
        "gopher" => Scheme::Gopher,
        "http"   => Scheme::Http,
        "https"  => Scheme::Http,
        _        => Scheme::Unknown,
    }
}
pub fn join_if_relative(base: &Url, url_str: &str) -> Result<Url, String> {
    match Url::parse(url_str) {
        Err(ParseError::RelativeUrlWithoutBase) => 
            match base.join(url_str) {
                Err(e) => Err(format!("{}", e)),
                Ok(url) => Ok(url),
            }
        Ok(url) => Ok(url),
        Err(e) => Err(format!("{}", e)),
    }
}
pub struct GemDoc {
    pub url: Url,
    pub status: Status,
    pub msg: String,
    pub doc: Vec<(GemType, String)>,
}
impl GemDoc {
    pub fn new(url: &Url) -> Self {
        let (response, content) = get_data(url).unwrap();
        let (status, msg) = parse_status(&response).unwrap();
        let doc = match status {
            Status::Success => parse_doc(&content, url),
            _ => vec![(GemType::Text, format!("Not a success"))],
        };
        Self {
            url: url.clone(),
            status: status,
            msg: msg,
            doc: doc,
        }
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
pub fn parse_doc(text_str: &str, source: &Url) -> Vec<(GemType, String)> {
    let mut vec = vec![];
    let mut preformat = false;
    for line in text_str.lines() {
        match line.split_at_checked(3) {
            Some(("```", _)) => preformat = !preformat,
            _ => match preformat {
                true => 
                    vec.push((GemType::PreFormat, line.to_string())),
                false => {
                    let (gem, text) = parse_formatted(line, source);
                    vec.push((gem, text.to_string()));
                }
            }
        }
    }
    vec
}
fn parse_formatted<'a>(line: &'a str, source: &Url) -> (GemType, &'a str) {
    // look for 3 character symbols
    if let Some((symbol, text)) = line.split_at_checked(3) {
        if symbol == "###" {
            return (GemType::HeadingThree, text)
        }
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = line.split_at_checked(2) {
        if symbol == "=>" {
            let (url_str, link_str) = util::split_whitespace_once(text);
            match join_if_relative(source, url_str) {
                Ok(url) =>
                    return (GemType::Link(parse_scheme(&url), url), link_str),
                Err(s) => 
                    return (GemType::BadLink(s), link_str)
            }
        } else if symbol == "##" {
            return (GemType::HeadingTwo, text)
        }
    }
    // look for 1 character symbols
    if let Some((symbol, text)) = line.split_at_checked(1) {
        if symbol == ">" {
            return (GemType::Quote, text)
        } else if symbol == "*" {
            return (GemType::ListItem, text)
        } else if symbol == "#" {
            return (GemType::HeadingOne, text)
        }
    }
    return (GemType::Text, line)
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
pub fn parse_status(line: &str) -> Result<(Status, String), String> {
    let (code, message) = util::split_whitespace_once(line);
    let status = getstatus(code.parse().unwrap()).unwrap();
    Ok((status, String::from(message)))
}
fn getstatus(code: u8) -> Result<Status, String> {
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
        _ => 
            Err(format!(
                "received status number {} which maps to nothing", 
                code)),
    }
}
// returns response and content
pub fn get_data(url: &Url) -> Result<(String, String), String> {
    let host = url.host_str().unwrap_or("");
    let urlf = format!("{}:1965", host);
    let failmsg = "Could not connect to ";

    // get connector
    let connector = TlsConnector::builder()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .or_else(|e| Err(format!("{}{}\n{}", failmsg, urlf, e)))?;

    // get socket address iterator
    let mut addrs_iter = urlf.to_socket_addrs()
        .or_else(|e| Err(format!("{}{}\n{}", failmsg, urlf, e)))?;

    // get socket address from socket address iterator
    let Some(socket_addr) = addrs_iter.next() 
        else {return Err(format!("Could not connect to {}", urlf))};

    // get tcp stream from socket address
    let tcpstream = 
        TcpStream::connect_timeout(&socket_addr, Duration::new(10, 0))
        .or_else(|e| Err(format!("Could not connect to {}\n{}", urlf, e)))?;

    // get stream from tcp stream
    let mut stream = connector.connect(&host, tcpstream) 
        .or_else(|e| Err(format!("Could not connect to {}\n{}", urlf, e)))?;

    // write url to stream
    stream.write_all(format!("{}\r\n", url).as_bytes())
        .or_else(|e| Err(format!("Could not write to {}\n{}", url, e)))?;

    // initialize response vector
    let mut response = vec![];

    // load response vector from stream
    stream.read_to_end(&mut response)
        .or_else(|e| Err(format!("Could not read {}\n{}", url, e)))?;

    // find clrf in response vector
    let Some(clrf_idx) = find_clrf(&response) 
        else {return Err("Could not find the clrf".to_string())};

    // separate response from content
    let content = response.split_off(clrf_idx + 2);

    // convert to String
    let content  = String::from_utf8_lossy(&content).to_string();
    let response = String::from_utf8_lossy(&response).to_string();
    Ok((response, content))
}
fn find_clrf(data: &[u8]) -> Option<usize> {
    let clrf = b"\r\n";
    data.windows(clrf.len()).position(|window| window == clrf)
}
