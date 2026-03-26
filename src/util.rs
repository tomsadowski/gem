// src/util.rs

use crossterm::{
  style::Color,
  event::KeyCode,
};
use toml::{Table, Value};


pub fn wrap_lines(text: &str, width: usize) -> Vec<String> {
  text
    .lines()
    .map(|l| wrap(l, width))
    .flatten()
    .collect()
}
pub fn wrap(text: &str, width: usize) -> Vec<String> {
  let mut idx = usize::MIN;
  let mut vec: Vec<String> = vec![];
  while idx < text.len() {
    let line = to_last_space(&text[idx..], width);
    idx += line.len();
    vec.push(line);
  }
  vec
}
pub fn to_last_space(text: &str, width: usize) -> String {
  if text.len() <= width {
    text.into()
  } else {
    let s: String = text[..width]
      .chars()
      .rev()
      .skip_while(|c| !c.is_whitespace())
      .collect();
    if s.len() == 0 {
      text[..width].into()
    } else {
      s.chars().rev().collect()
    }
  }
}
pub fn parse_color(value: &Value) -> Result<Color, String> {
  if let Value::String(hex) = value {
    color_from_hex(&hex)
  } else {
    Err("could not parse color from value".into())
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
