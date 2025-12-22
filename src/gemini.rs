// gem/src/gemini
// frontend agnostic
use std::{
    time::{Duration}, 
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs}};
use url::{
    Url, ParseError};
use native_tls::TlsConnector;

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
    let (code, message) = 
        match line.trim().split_once(' ') {
            Some((c, msg)) => 
                (c.trim(), msg.trim()),
            None => 
                (line.trim(), ""),
    };
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
    let (url_str, text) = match line.trim().split_once(' ') {
        Some((link, txt)) => (link.trim(), txt.trim()),
        None => (line.trim(), line.trim()),
    };
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
