// src/util.rs

use crossterm::{
  style::Color,
  event::KeyCode,
};
use toml::{Table, Value};


pub fn wrap_lines(text: &str, width: usize) -> Vec<String> {
  text.lines().map(|l| wrap(l, width)).flatten().collect()
}
pub fn wrap(text: &str, width: usize) -> Vec<String> {
  let mut idx = usize::MIN;
  let mut vec: Vec<String> = vec![];
  while idx < text.len() {
    let line: String = {
      let text = &text[idx..];
      if text.len() <= width {
        text.into()
      } else {
        // search for first whitespace from right
        let s: String = text[..width].chars().rev()
          .skip_while(|c| !c.is_whitespace()).collect();
        // no space found, return whole slice
        if s.len() == 0 {
          text[..width].into()
        // space found, return up to that space
        } else {
          s.chars().rev().collect()
        }
      }
    };
    idx += line.len();
    vec.push(line);
  }
  vec
}
pub fn parse_color(v: &Value) -> Result<Color, String> {
  if let Value::String(s) = v {
    fn try_hex(c: char) -> Result<u8, String> {
      match c {
        '0' => Ok(0),  
        '1' => Ok(1),  
        '2' => Ok(2),  
        '3' => Ok(3),
        '4' => Ok(4),  
        '5' => Ok(5),  
        '6' => Ok(6),  
        '7' => Ok(7),
        '8' => Ok(8),  
        '9' => Ok(9),  
        'a' => Ok(10), 
        'b' => Ok(11),
        'c' => Ok(12), 
        'd' => Ok(13), 
        'e' => Ok(14), 
        'f' => Ok(15),
        _   => Err(format!("{} is not a hex character", c)),
      }
    }
    let mut c = s.chars();
    let r1 = c
      .next()
      .ok_or("missing first red".into())
      .and_then(|c| try_hex(c))?;
    let r2 = c
      .next()
      .ok_or("missing second red".into())
      .and_then(|c| try_hex(c))?;
    let g1 = c
      .next()
      .ok_or("missing first green".into())
      .and_then(|c| try_hex(c))?;
    let g2 = c
      .next()
      .ok_or("missing second green".into())
      .and_then(|c| try_hex(c))?;
    let b1 = c
      .next()
      .ok_or("missing first blue".into())
      .and_then(|c| try_hex(c))?;
    let b2 = c
      .next()
      .ok_or("missing second blue".into())
      .and_then(|c| try_hex(c))?;
    Ok(Color::Rgb {
      r: 16 * r1 + r2, 
      g: 16 * g1 + g2, 
      b: 16 * b1 + b2
    })
  } else {
    Err(format!("could not parse color from value {}", v))
  }
}
