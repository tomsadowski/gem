// gem/src/gemini
// frontend agnostic
use crate::util;
use url::{Url, ParseError};

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
pub fn parse_status(line: &str) -> Result<(Status, String), String> {
    let (code, message) = util::split_whitespace_once(line);
    let status = getstatus(code.parse().unwrap()).unwrap();
    // return Result
    Ok((status, String::from(message)))
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
    match line.split_at_checked(3) {
        Some(("```", _)) => true,
        _ => false,
    }
}
fn parse_formatted(line: &str) -> Result<(GemTextData, String), String> {
    // look for 3 character symbols
    if let Some((symbol, text)) = line.split_at_checked(3) {
        if symbol == "###" {
            return Ok((GemTextData::HeadingThree, text.to_string()))
        }
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = line.split_at_checked(2) {
        if symbol == "=>" {
            let (url, text) = parse_scheme(text)
                .or_else(
                    |e| Err(format!("could not parse link {:?}", e)))?;
            return Ok((GemTextData::Link(url), text))
        }
        if symbol == "##" {
            return Ok((GemTextData::HeadingTwo, text.to_string()))
        }
    }
    // look for 1 character symbols
    if let Some((symbol, text)) = line.split_at_checked(1) {
        if symbol == ">" {
            return Ok((GemTextData::Quote, text.to_string()))
        }
        if symbol == "*" {
            return Ok((GemTextData::ListItem, text.to_string()))
        }
        if symbol == "#" {
            return Ok((GemTextData::HeadingOne, text.to_string()))
        }
    }
    return Ok((GemTextData::Text, line.to_string()))
}
fn parse_scheme(line: &str) -> Result<(Scheme, String), String> {
    let (url_str, text) = util::split_whitespace_once(line);
    let url_result = Url::parse(url_str);
    if let Ok(url) = url_result {
       let scheme = match url.scheme() {
            "gemini" => Scheme::Gemini(url),
            "gopher" => Scheme::Gopher(url),
            "http"   => Scheme::Http(url),
            "https"  => Scheme::Http(url),
            _        => Scheme::Unknown(url),
        };
        Ok((scheme, String::from(text)))
    } else if Err(ParseError::RelativeUrlWithoutBase) == url_result {
        Ok((Scheme::Relative(String::from(url_str)), String::from(text)))
    } else {
        Err(format!("no parse url")) 
    }
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
