// gem/src/gemini
// frontend agnostic

use native_tls::{
    TlsConnector
};
use std::{
    time::{Duration}, 
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};
use url::{
    Url, ParseError
};
use regex::{
    Regex
};

const GOPHER_SCHEME: &str    = "gopher";
const HTTPS_SCHEME: &str     = "https";
const HTTP_SCHEME: &str      = "http";
const LINK_SYMBOL: &str      = "=>";
const TOGGLE_SYMBOL: &str    = "```";
const QUOTE_SYMBOL: &str     = ">";
const LIST_ITEM_SYMBOL: &str = "*";
const HEADING_1_SYMBOL: &str = "#";
const HEADING_2_SYMBOL: &str = "##";
const HEADING_3_SYMBOL: &str = "###";
const GEMINI_PORT: &str      = "1965";
const GEMINI_SCHEME: &str    = "gemini";
const LINK_REGEX: &str       = r"^\s*(\S*)\s*(.*)?$";
const STATUS_REGEX: &str = r"^(\d{1,3})[ \t](.*)\r\n$";

#[derive(Clone, PartialEq, Debug)]
pub enum Scheme {
    Gemini(Url),
    Gopher(Url),
    Http(Url),
    Relative(String),
    Unknown(Url),
}
#[derive(Clone, PartialEq, Debug)]
pub enum GemTextData {
    HeadingOne,
    HeadingTwo,
    HeadingThree,
    Text, 
    PreFormat,
    Link(Scheme),
    ListItem,
    Quote,
} 
impl Scheme {
    fn from_str(line: &str) -> Result<(Scheme, String), String> {
        // get regex
        let Ok(regex) = Regex::new(LINK_REGEX)
            else {return Err(format!("regex: no parse"))};
        // get captures
        let Some(captures) = regex.captures(&line) 
            else {return Err(format!("regex: no captures"))};
        // get string
        let url_str = captures
            .get(1)
            .map_or("", |m| m.as_str())
            .to_string();
        // get result
        let url_result = Url::parse(&url_str);
        // get label 
        let label_str = captures
            .get(2)
            .map_or("", |m| m.as_str());
        let label = 
            if label_str.is_empty() {
                url_str.clone()
            } else {
                label_str.to_string()
            };
        // return Result
        if let Ok(url) = url_result {
            let scheme = match url.scheme() {
                GEMINI_SCHEME => Scheme::Gemini(url),
                GOPHER_SCHEME => Scheme::Gopher(url),
                HTTP_SCHEME   => Scheme::Http(url),
                HTTPS_SCHEME  => Scheme::Http(url),
                _             => Scheme::Unknown(url),
            };
            Ok((scheme, label))
        } else if Err(ParseError::RelativeUrlWithoutBase) == url_result {
            Ok((Scheme::Relative(url_str), label))
        } else {
            Err(format!("no parse url")) 
        }
    }
}
pub fn parse_doc(lines: Vec<&str>) 
    -> Result<Vec<(GemTextData, String)>, String> 
{
    let mut vec = vec![];
    let mut lines_iter = lines.iter();
    // return empty output if empty input
    let Some(first_line) = lines_iter.next() 
        else {return Ok(vec)};
    let mut preformat_flag = is_toggle(first_line);
    if !preformat_flag {
        // return error if cannot parse formatted line
        let formatted = parse_formatted(first_line)
            .or_else(|e| Err(format!("{}", e)))?;
        vec.push(formatted);
    }
    // parse remaining lines
    for line in lines_iter {
        if is_toggle(line) {
            preformat_flag = !preformat_flag;
        } else if preformat_flag {
            vec.push((GemTextData::PreFormat, line.to_string()));
        } else {
            let formatted = parse_formatted(line)
                .or_else(|e| Err(format!("{}", e)))?;
            vec.push(formatted);
        }
    }
    Ok(vec)
}
fn is_toggle(line: &str) -> bool {
    if let Some((symbol, _text)) = line.split_at_checked(3) {
        if symbol == TOGGLE_SYMBOL {
            return true
        }
    }
    return false
}
fn parse_formatted(line: &str) -> Result<(GemTextData, String), String> {
    // look for 3 character symbols
    if let Some((symbol, text)) = line.split_at_checked(3) {
        if symbol == HEADING_3_SYMBOL {
            return Ok((GemTextData::HeadingThree, text.to_string()))
        }
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = line.split_at_checked(2) {
        if symbol == LINK_SYMBOL {
            let (url, text) = Scheme::from_str(text)
                .or_else(
                    |e| Err(format!("could not parse link {:?}", e)))?;
            return Ok((GemTextData::Link(url), text))
        }
        if symbol == HEADING_2_SYMBOL {
            return Ok((GemTextData::HeadingTwo, text.to_string()))
        }
    }
    // look for 1 character symbols
    if let Some((symbol, text)) = line.split_at_checked(1) {
        if symbol == QUOTE_SYMBOL {
            return Ok((GemTextData::Quote, text.to_string()))
        }
        if symbol == LIST_ITEM_SYMBOL {
            return Ok((GemTextData::ListItem, text.to_string()))
        }
        if symbol == HEADING_1_SYMBOL {
            return Ok((GemTextData::HeadingOne, text.to_string()))
        }
    }
    return Ok((GemTextData::Text, line.to_string()))
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
pub fn getstatus(code: i16) -> Result<Status, String> {
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
pub fn getstatustuple(line: &str) -> Result<(Status, String), String> {
    // get regex
    let Ok(regex) = Regex::new(STATUS_REGEX)
        else {return Err("".to_string())};
    // get captures
    let Some(captures) = regex.captures(&line) 
        else {return Err("".to_string())};
    // get code from captures
    let Ok(code) = captures
        .get(1)
        .map_or("", |m| m.as_str())
        .parse()
        else {return Err("".to_string())};
    // get meta from captures
    let meta = captures
        .get(2)
        .map_or("", |m| m.as_str())
        .to_string();

    let status = getstatus(code).unwrap();
    // return Result
    Ok((status, meta))
}
// returns response and content
pub fn get_data(url: &Url) -> Result<(String, String), String> {
    let host = url.host_str().unwrap_or("");
    let urlf = format!("{}:{}", host, GEMINI_PORT);
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
    let tcpstream = TcpStream::connect_timeout
        (&socket_addr, Duration::new(10, 0))
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
