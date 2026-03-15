// src/util.rs

use std::{
  time::{Duration}, 
  io::{Write, Read},
  net::{TcpStream, ToSocketAddrs},
};
use crossterm::{
  style::{Color},
};
use url::{Url, ParseError};
use toml::{Value};
use native_tls::TlsConnector;

pub fn parse_color(value: &Value) -> Result<Color, String> {
  if let Value::String(hex) = value {
    color_from_hex(&hex)

  } else {
    Err("fart".into())
  }
}

pub fn try_hex_from_char(u: char) -> Option<u8> {
  match u {
    '0' => Some(0),
    '1' => Some(1),
    '2' => Some(2),
    '3' => Some(3),
    '4' => Some(4),
    '5' => Some(5),
    '6' => Some(6),
    '7' => Some(7),
    '8' => Some(8),
    '9' => Some(9),
    'a' => Some(10),
    'b' => Some(11),
    'c' => Some(12),
    'd' => Some(13),
    'e' => Some(14),
    'f' => Some(15),
    _ => None,
  }
}

pub fn try_next_u8<I>(v: &mut I) -> Option<u8> 
where I: Iterator<Item = char>
{
  let a = v.next().and_then(|c| try_hex_from_char(c));
  let b = v.next().and_then(|c| try_hex_from_char(c));
  a.zip(b).map(|(a, b)| 16 * a + b)
}

pub fn color_from_hex(text: &str) -> Result<Color, String> {
  let mut c = text.chars();

  let r = try_next_u8(&mut c);
  let g = try_next_u8(&mut c);
  let b = try_next_u8(&mut c);

  match (r, g, b) {
    (Some(r), Some(g), Some(b)) => {
      Ok(Color::Rgb {r, g, b})
    }
    _ => {
      Err("this... is not hex".into())
    }
  }
}

pub fn u16_or_0(u: usize) -> u16 {
  u16::try_from(u).unwrap_or(u16::MIN)
}

pub fn u16_or_max(u: usize) -> u16 {
  u16::try_from(u).unwrap_or(u16::MAX)
}

pub fn split_whitespace_once(source: &str) 
  -> (&str, &str) 
{
  let line = source.trim();

  if let Some(i) = line.find("\u{0009}") {
    (line[..i].trim(), line[i..].trim())

  } else if let Some(i) = line.find(" ") {
    (line[..i].trim(), line[i..].trim())

  } else {(line, line)}
}

pub fn wrap(line: &str, width: usize) -> Vec<String> {

  let length = line.len();
  let mut wrapped: Vec<String> = vec![];

  // assume slice bounds
  let mut start = 0;
  let mut end = width;

  while end < length {

    start = line.ceil_char_boundary(start);
    end   = line.floor_char_boundary(end);
    let longest = &line[start..end];

    // try to break line at a space
    match longest.rsplit_once(' ') {

      // there is a space to break on
      Some((a, b)) => {
        let shortest = match a.len() {
          0 => b, 
          _ => a
        };

        wrapped.push(String::from(shortest.trim()));
        start += shortest.len();
        end = start + width;
      }

      // there is no space to break on
      None => {
        wrapped.push(String::from(longest.trim()));
        start = end;
        end += width;
      }
    }
  }

  // add the remaining text
  if start < length {
    start = line.floor_char_boundary(start);
    wrapped.push(String::from(line[start..].trim()));
  }

  wrapped
}

pub fn join_if_relative(base: &Url, url_str: &str) 
  -> Result<Url, ParseError> 
{
  Url::parse(url_str).or_else(|e|

    if let ParseError::RelativeUrlWithoutBase = e {
      base.join(url_str)

    } else {
      Err(e)
    }
  )
}

#[derive(Clone, PartialEq, Debug)]
pub enum Scheme {
  Gemini, 
  Gopher, 
  Http, 
  Unknown
}
impl From<&Url> for Scheme {
  fn from(url: &Url) -> Scheme {
    match url.scheme() {
      "gemini" => Scheme::Gemini,
      "gopher" => Scheme::Gopher,
      "http"   => Scheme::Http,
      "https"  => Scheme::Http,
      _        => Scheme::Unknown,
    }
  }
}

// returns response and content
pub fn get_data(url: &Url) 
  -> Result<(String, String), String> 
{
  let host = url.host_str().unwrap_or("");
  let urlf = format!("{}:1965", host);

  // get connector
  let connector = TlsConnector::builder()
    .danger_accept_invalid_hostnames(true)
    .danger_accept_invalid_certs(true)
    .build()
    .map_err(|e| e.to_string())?;

  // get socket address iterator
  let mut addrs_iter = urlf.to_socket_addrs()
    .map_err(|e| e.to_string())?;

  // get socket address from socket address iterator
  let Some(socket_addr) = addrs_iter.next() 
    else {return Err(format!("{}", urlf))};

  // get tcp stream from socket address
  let tcpstream = 
    TcpStream::connect_timeout(
      &socket_addr, Duration::new(10, 0))
    .map_err(|e| e.to_string())?;

  // get stream from tcp stream
  let mut stream = connector.connect(&host, tcpstream) 
    .map_err(|e| e.to_string())?;

  // write url to stream
  stream.write_all(format!("{}\r\n", url).as_bytes())
    .map_err(|e| e.to_string())?;
  
  // read into response vector
  let mut response = vec![];
  stream.read_to_end(&mut response)
    .map_err(|e| e.to_string())?;

  // separate response from content
  let clrf = b"\r\n";
  let content = response
    .windows(clrf.len())
    .position(|window| window == clrf)
    .map(|idx| response.split_off(idx + 2))
    .map(|content| String::from_utf8_lossy(&content)
      .to_string())
    .unwrap_or(String::from("no content"));

  // convert to String
  let response = String::from_utf8_lossy(&response)
    .to_string();

  Ok((response, content))
}
