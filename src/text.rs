// src/text.rs

use crate::util;

pub trait Linear {
  fn len(&self) -> usize;
  fn head(&self) -> usize;
  fn head_mut(&mut self) -> &mut usize;
  fn max_head(&self) -> usize;

  fn fit(&mut self, new_cursor: usize) {
    *self.head_mut() = self.max_head().min(new_cursor);
  }
  fn start(&mut self) {
    *self.head_mut() = 0;
  }
  fn end(&mut self) {
    *self.head_mut() = self.max_head();
  }
  fn peek_backward(&self, mut step: usize) -> usize {
    if step > self.head() {
      step - self.head()
    } else {0}
  }
  fn peek_forward(&self, mut step: usize) -> usize {
    let max_head = self.max_head();
    if self.head() + step > max_head {
      self.head() + step - max_head
    } else {0}
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
  fn tight_forward(&mut self, mut step: usize) -> usize {
    let max_head = self.len().saturating_sub(1);
    if self.head() + step > max_head {
      step = self.head() + step - max_head;
      *self.head_mut() = max_head;
      step
    } else {
      *self.head_mut() += step;
      0
    }
  }
  fn forward(&mut self, mut step: usize) -> usize {
    let max_head = self.max_head();
    if self.head() + step > max_head {
      step = self.head() + step - max_head;
      *self.head_mut() = max_head;
      step
    } else {
      *self.head_mut() += step;
      0
    }
  }
}
#[derive(Clone, Debug, Default)]
pub struct TextLine {
  pub head: usize,
  pub text: String,
}
impl From<&str> for TextLine {
  fn from(item: &str) -> Self {
    Self {head: 0, text: item.into()}
  }
}
impl Linear for TextLine {
  fn max_head(&self) -> usize {
    self.text.len()
  }
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
impl TextLine {
  pub fn delete(&mut self) -> bool {
    if self.peek_forward(1) == 0 || self.text.len() != 0 {
      self.text.remove(self.head);
      true
    } else {false}
  }
  pub fn backspace(&mut self) -> bool {
    if self.peek_backward(1) == 0 {
      self.backward(1);
      self.text.remove(self.head);
      true
    } else {false}
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

pub trait Planar {
  fn x_len(&self) -> usize;
  fn x_head(&self) -> usize;
  fn y_len(&self) -> usize;
  fn y_head(&self) -> usize;
  fn y_head_mut(&mut self) -> &mut usize;
}
impl<P: Planar> Linear for P {
  fn len(&self) -> usize {
    self.y_len()
  }
  fn max_head(&self) -> usize {
    self.y_len().saturating_sub(1)
  }
  fn head(&self) -> usize {
    self.y_head()
  }
  fn head_mut(&mut self) -> &mut usize {
    self.y_head_mut()
  }
}
#[derive(Clone, Default)]
pub struct TextPlane {
  pub head: usize,
  pub text: Vec<TextLine>,
}
impl Planar for TextPlane {
  fn x_len(&self) -> usize {
    self.text[self.head].len()
  }
  fn x_head(&self) -> usize {
    self.text[self.head].head()
  }
  fn y_len(&self) -> usize {
    self.text.len()
  }
  fn y_head(&self) -> usize {
    self.head
  }
  fn y_head_mut(&mut self) -> &mut usize {
    &mut self.head
  }
}
impl TextPlane {
  pub fn delete(&mut self) -> bool {
    if !self.text[self.head].delete() {
      if self.forward(1) == 0 {
        self.text[self.head].start();
        self.delete() 
      } else {false} 
    } else {true}
  }
  pub fn backspace(&mut self) -> bool {
    if !self.text[self.head].backspace() {
      if self.backward(1) == 0 {
        self.text[self.head].end();
        self.backspace() 
      } else {false}
    } else {true}
  }
  pub fn insert(&mut self, c: char) -> bool {
    self.text[self.head].insert(c)
  }
  pub fn new(text: &str, width: u16) -> Self {
    let text: Vec<TextLine> = 
      util::wrap_lines(text, width.into())
        .iter()
        .map(|line| TextLine::from(line.as_str()))
        .collect();
    Self {head: 0, text}
  }
  pub fn get_idx(&self) -> usize {
    self.text[..self.head()]
      .iter()
      .map(|line| line.len())
      .chain(std::iter::once(self.x_head()))
      .sum()
  }
  pub fn resize(&mut self, text: &str, width: u16) {
    let idx = self.get_idx();
    self.text = 
      util::wrap_lines(text, width.into())
        .into_iter()
        .map(|line| TextLine::from(line.as_str()))
        .collect();
    self.start();
    self.text[self.head].start();
    self.restore_idx(idx);
  }
  pub fn up(&mut self, step: usize) -> bool {
    let x = self.x_head();
    if self.backward(step) == 0 {
      self.text[self.head].fit(x);
      true
    } else {false}
  }
  pub fn down(&mut self, step: usize) -> bool {
    let x = self.x_head();
    if self.forward(step) == 0 {
      self.text[self.head].fit(x);
      true
    } else {false}
  }
  pub fn left(&mut self, step: usize) -> usize {
    let remainder = self.text[self.head].backward(step);
    if remainder == 0 {
      0
    } else if self.backward(1) == 0 {
      self.text[self.head].end();
      self.left(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
  pub fn restore_idx(&mut self, step: usize) -> usize {
    let remainder = self.text[self.head].tight_forward(step);
    if remainder == 0 {
      0
    } else if self.forward(1) == 0 {
      self.text[self.head].start();
      self.restore_idx(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
  pub fn right(&mut self, step: usize) -> usize {
    let remainder = self.text[self.head].forward(step);
    if remainder == 0 {
      0
    } else if self.forward(1) == 0 {
      self.text[self.head].start();
      self.right(remainder.saturating_sub(1))
    } else {
      remainder
    }
  }
}
