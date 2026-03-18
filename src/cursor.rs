// src/cursor.rs

pub trait Cursor {

  fn len(&self) -> usize;

  fn cursor(&mut self) -> &mut usize;

  fn get_cursor(&self) -> usize;

  fn max(&self) -> usize {
    self.len().saturating_sub(1)
  }

  fn fit(&mut self, new_cursor: usize) {
    *self.cursor() = self.max().min(new_cursor);
  }

  fn move_to_start(&mut self) {
    *self.cursor() = 0;
  }

  fn move_to_end(&mut self) {
    *self.cursor() = self.max();
  }

  fn try_backward(&mut self, step: usize) -> bool {
    let cursor = self.get_cursor();
    if step > cursor {
      false
    } else {
      *self.cursor() -= step;
      true
    }
  }

  fn try_forward(&mut self, step: usize) -> bool {
    let cursor = self.get_cursor();
    if cursor + step > self.max() {
      false

    } else {
      *self.cursor() += step;
      true
    }
  }

  fn get_wrap_backward(&self, mut step: usize) -> usize {
    let cursor = self.get_cursor();
    if step > cursor {
      step - cursor
    } else {
      0
    }
  }

  fn get_wrap_forward(&self, mut step: usize) -> usize {
    let max = self.max();
    let cursor = self.get_cursor();
    if cursor + step > max {
      cursor + step - max
    } else {
      0
    }
  }

  fn wrap_backward(&mut self, mut step: usize) -> usize {
    let cursor = self.get_cursor();
    if step > cursor {
      step -= cursor;
      *self.cursor() = 0;
      step
    } else {
      *self.cursor() -= step;
      0
    }
  }

  fn wrap_forward(&mut self, mut step: usize) -> usize {
    let max = self.max();
    let cursor = self.get_cursor();
    if cursor + step > max {
      step = cursor + step - max;
      *self.cursor() = max;
      step
    } else {
      *self.cursor() += step;
      0
    }
  }
}

#[derive(Clone)]
pub struct ScCurs {
  pub scroll: usize,
  pub cursor: u16,
}
impl ScCurs {

  pub fn update<C>(&mut self, ctext: &C, rng: &ScRange) 
    -> bool
  where C: Cursor
  {
    let cursor = usize::from(self.cursor);
    let idx = ctext.get_cursor(); 

    // move forward
    if cursor + self.scroll < idx {
      true

    // move backward
    } else if cursor + self.scroll > idx {
      true

    // no change
    } else {
      false
    }
  }
}

#[derive(Clone)]
pub struct ScRange {
  pub scroll_start: u16,
  pub scroll_end: u16,
  pub start: u16,
  pub end:   u16,
}
impl Default for ScRange {
  fn default() -> Self {
    Self {
      scroll_start: 0, 
      scroll_end: 0, 
      start: 0, 
      end: 0
    }
  }
}
impl ScRange {
}

#[derive(Clone)]
pub struct CursorText {
  pub cursor: usize,
  pub text: String,
}
impl Default for CursorText {
  fn default() -> Self {
    Self {cursor: 0, text: "".to_string()}
  }
}
impl From<&str> for CursorText {
  fn from(item: &str) -> Self {
    Self {cursor: 0, text: item.into()}
  }
}
impl Cursor for CursorText {
  fn len(&self) -> usize {
    self.text.len()
  }
  fn cursor(&mut self) -> &mut usize {
    &mut self.cursor
  }
  fn get_cursor(&self) -> usize {
    self.cursor
  }
}
impl CursorText {

  pub fn delete(&mut self) -> bool {
    if self.get_wrap_forward(1) != 0 ||
      self.text.len() == 0
    {
      false

    } else {
      self.text.remove(self.cursor);
      true
    }
  }

  pub fn backspace(&mut self) -> bool {
    if self.get_wrap_backward(1) != 0 {
      false

    } else {
      self.wrap_backward(1);
      self.text.remove(self.cursor);
      true
    }
  }

  pub fn insert(&mut self, c: char) -> bool {
    if self.cursor + 1 == self.text.len() || 
      self.text.len() == 0 
    {
      self.text.push(c);
      self.wrap_forward(1);
      true

    } else {
      self.text.insert(self.cursor, c);
      self.wrap_forward(1);
      true
    }
  }
}

#[derive(Clone)]
pub struct CursorDoc<T> {
  pub cursor: usize,
  pub text: Vec<T>,
}
impl<T> Default for CursorDoc<T> 
where T: Default
{
  fn default() -> Self {
    Self {cursor: 0, text: vec![T::default()]}
  }
}
impl<T> Cursor for CursorDoc<T> {
  fn len(&self) -> usize {
    self.text.len()
  }
  fn cursor(&mut self) -> &mut usize {
    &mut self.cursor
  }
  fn get_cursor(&self) -> usize {
    self.cursor
  }
}
impl<T> CursorDoc<T> 
where T: Cursor
{
  pub fn move_up(&mut self, step: usize) -> bool {
    let x = self.text[self.cursor].get_cursor();
    if self.wrap_backward(step) == 0 {
      self.text[self.cursor].fit(x);
      true
    } else {
      false
    }
  }

  pub fn move_down(&mut self, step: usize) -> bool {
    let x = self.text[self.cursor].get_cursor();
    if self.wrap_forward(step) == 0 {
      self.text[self.cursor].fit(x);
      true
    } else {
      false
    }
  }

  pub fn move_left(&mut self, step: usize) -> bool {
    self.wrap_left(step) == step
  }

  pub fn move_right(&mut self, step: usize) -> bool {
    self.wrap_right(step) == step
  }

  pub fn wrap_left(&mut self, step: usize) -> usize {
    let remainder = self
      .text[self.cursor].wrap_backward(step);

    // no wrapping required
    if remainder == 0 {
      0
    // try going up
    } else if self.wrap_backward(1) == 0 {
      self.text[self.cursor].move_to_end();
      self.wrap_left(remainder)

    } else {
      remainder
    }
  }

  pub fn wrap_right(&mut self, step: usize) -> usize {
    let remainder = self
      .text[self.cursor].wrap_forward(step);

    // no wrapping required
    if remainder == 0 {
      0
    // try going down
    } else if self.wrap_forward(1) == 0 {
      self.text[self.cursor].move_to_start();
      self.wrap_right(remainder)

    } else {
      remainder
    }
  }
}
