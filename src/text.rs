// src/text.rs

use std::iter;

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
pub trait Tape {
  fn len(&self) -> usize;
  fn head(&self) -> usize;
  fn head_mut(&mut self) -> &mut usize;
  fn max(&self) -> usize {
    self.len().saturating_sub(1)
  }
  fn fit(&mut self, new_cursor: usize) {
    *self.head_mut() = self.max().min(new_cursor);
  }
  fn move_to_start(&mut self) {
    *self.head_mut() = 0;
  }
  fn move_to_end(&mut self) {
    *self.head_mut() = self.max();
  }
  fn peek_backward(&self, mut step: usize) -> usize {
    if step > self.head() {
      step - self.head()
    } else {
      0
    }
  }
  fn peek_forward(&self, mut step: usize) -> usize {
    let max = self.max();
    if self.head() + step > max {
      self.head() + step - max
    } else {
      0
    }
  }
  fn backward(&mut self, mut step: usize) -> usize {
    if step > self.head() {
      step -= self.head();
      *self.head_mut() = 0;
      step
    } else {
      *self.head_mut() -= step;
      0
    }
  }
  fn forward(&mut self, mut step: usize) -> usize {
    let max = self.max();
    if self.head() + step > max {
      step = self.head() + step - max;
      *self.head_mut() = max;
      step
    } else {
      *self.head_mut() += step;
      0
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct TextTape {
  pub head: usize,
  pub text: String,
}
impl From<&str> for TextTape {
  fn from(item: &str) -> Self {
    Self {head: 0, text: item.into()}
  }
}
impl Tape for TextTape {
  fn len(&self) -> usize {
    self.text.len()
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
}
impl TextTape {
  pub fn delete(&mut self) -> bool {
    if self.peek_forward(1) != 0 || self.text.len() == 0 {
      false
    } else {
      self.text.remove(self.head);
      true
    }
  }
  pub fn backspace(&mut self) -> bool {
    if self.peek_backward(1) != 0 {
      false
    } else {
      self.backward(1);
      self.text.remove(self.head);
      true
    }
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.head + 1 == self.text.len() || self.text.len() == 0 {
      self.text.push(c);
      self.forward(1);
      true
    } else {
      self.text.insert(self.head, c);
      self.forward(1);
      true
    }
  }
}

#[derive(Clone, Default)]
pub struct CursorDoc {
  pub head: usize,
  pub text: Vec<TextTape>,
}
impl Tape for CursorDoc {
  fn len(&self) -> usize {
    self.text.len()
  }
  fn head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
  fn head(&self) -> usize {
    self.head
  }
}
impl CursorDoc {
  pub fn x_len(&self) -> usize {
    self.text[self.head()].len()
  }
  pub fn y_len(&self) -> usize {
    self.len()
  }
  pub fn x(&self) -> usize {
    self.text[self.head()].head()
  }
  pub fn y(&self) -> usize {
    self.head()
  }
  pub fn new(text: &str, width: u16) -> Self {
    let text: Vec<TextTape> = 
      wrap_lines(text, width.into())
        .iter()
        .map(|line| TextTape::from(line.as_str()))
        .collect();
    Self {head: 0, text}
  }
  pub fn flat_head(&self) -> usize {
    self.text[..self.head()]
      .iter()
      .map(|line| line.len())
      .chain(iter::once(self.x()))
      .sum()
  }
  pub fn resize(&mut self, text: &str, width: u16) {
    let idx = self.flat_head();
    self.text = 
      wrap_lines(text, width.into())
        .into_iter()
        .map(|line| TextTape::from(line.as_str()))
        .collect();
    self.text[0].head = 0;
    self.head = 0;
    self.right(idx);
  }
  pub fn up(&mut self, step: usize) -> bool {
    let x = self.text[self.head].head();
    if self.backward(step) == 0 {
      self.text[self.head].fit(x);
      true
    } else {
      false
    }
  }
  pub fn down(&mut self, step: usize) -> bool {
    let x = self.text[self.head].head();
    if self.forward(step) == 0 {
      self.text[self.head].fit(x);
      true
    } else {
      false
    }
  }
  pub fn left(&mut self, step: usize) -> usize {
    let remainder = self.text[self.head].backward(step);
    if remainder == 0 {
      0
    } else if self.backward(1) == 0 {
      self.text[self.head].move_to_end();
      self.left(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
  pub fn right(&mut self, step: usize) -> usize {
    let remainder = self.text[self.head].forward(step);
    if remainder == 0 {
      0
    } else if self.forward(1) == 0 {
      self.text[self.head].move_to_start();
      self.right(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
}
