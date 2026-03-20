// src/cursor.rs

pub trait Tape {
  // required
  fn len(&self) -> usize;

  fn head(&self) -> usize;

  fn head_mut(&mut self) -> &mut usize;

  // provided
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
  fn try_backward(&mut self, step: usize) -> bool {
    if step > self.head() {
      false
    } else {
      *self.head_mut() -= step;
      true
    }
  }
  fn try_forward(&mut self, step: usize) -> bool {
    if self.head() + step > self.max() {
      false
    } else {
      *self.head_mut() += step;
      true
    }
  }
  fn backward_remainder(&self, mut step: usize) -> usize {
    if step > self.head() {
      step - self.head()
    } else {
      0
    }
  }
  fn forward_remainder(&self, mut step: usize) -> usize {
    let max = self.max();
    if self.head() + step > max {
      self.head() + step - max
    } else {
      0
    }
  }
  fn wrap_backward(&mut self, mut step: usize) -> usize {
    if step > self.head() {
      step -= self.head();
      *self.head_mut() = 0;
      step
    } else {
      *self.head_mut() -= step;
      0
    }
  }
  fn wrap_forward(&mut self, mut step: usize) -> usize {
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

#[derive(Clone, Debug)]
pub struct SpanPos {
  pub scroll: usize,
  pub cursor: u16,
}
impl Default for SpanPos {
  fn default() -> Self {
    Self {scroll: 0, cursor: 0}
  }
}
impl From<&ScreenSpan> for SpanPos {
  fn from(item: &ScreenSpan) -> Self {
    Self {
      scroll: 0,
      cursor: item.start,
    }
  }
}
impl SpanPos {
  pub fn head(&self) -> usize {
    self.scroll + usize::from(self.cursor)
  }
}

#[derive(Clone, Debug)]
pub struct ScreenSpan {
  pub scroll_start: usize,
  pub scroll_end: usize,
  pub start: u16,
  pub end:   u16,
}
impl Default for ScreenSpan {
  fn default() -> Self {
    Self {
      scroll_start: 0, 
      scroll_end: 0, 
      start: 0, 
      end: 0
    }
  }
}
impl ScreenSpan {
  pub fn new_span_pos<T>(&self, tape: &T) -> SpanPos
  where T: Tape
  {
    let scroll = tape.head()
      .saturating_sub(self.inv_end_width());

    let cursor = 
      usize::from(self.start) + 
        tape.head() - scroll;

    SpanPos {
      scroll,
      cursor: u16::try_from(cursor).unwrap_or(u16::MIN)
    }
  }
  //    |...(___)___|
  pub fn start_width(&self) -> usize {
    self.scroll_start - usize::from(self.start)
  }
  //    |___(...)___|
  pub fn scroll_width(&self) -> usize {
    self.scroll_end - self.scroll_start
  }
  //    |___(___)...|
  pub fn end_width(&self) -> usize {
    usize::from(self.end) - self.scroll_end
  }
  //    |...(...)...|
  pub fn width(&self) -> usize {
    usize::from(self.end) - usize::from(self.start)
  }
  //    |...(...)___|
  pub fn inv_end_width(&self) -> usize {
    self.scroll_end - usize::from(self.start)
  }
  //    |___(...)...|
  pub fn inv_start_width(&self) -> usize {
    usize::from(self.end) - self.scroll_start
  }
}

#[derive(Clone, Debug, Default)]
pub struct ScreenPos {
  pub x: SpanPos,
  pub y: SpanPos,
}
impl ScreenPos {
  pub fn x_cursor(&self) -> u16 {
    self.x.cursor
  }
  pub fn y_cursor(&self) -> u16 {
    self.y.cursor
  }
  pub fn x_scroll(&self) -> usize {
    self.x.scroll
  }
  pub fn y_scroll(&self) -> usize {
    self.y.scroll
  }
}

#[derive(Clone, Debug, Default)]
pub struct Screen {
  pub x: ScreenSpan,
  pub y: ScreenSpan,
}
impl Screen {
  pub fn new_screen_pos<T>(&self, tape: &T) -> ScreenPos 
  where T: Tape
  {
    ScreenPos {
      x: self.x.new_span_pos(tape),
      y: self.y.new_span_pos(tape),
    }
  }
}

#[derive(Clone, Debug)]
pub struct CursorText {
  pub head: usize,
  pub text: String,
}
impl Default for CursorText {
  fn default() -> Self {
    Self {head: 0, text: "".to_string()}
  }
}
impl From<&str> for CursorText {
  fn from(item: &str) -> Self {
    Self {head: 0, text: item.into()}
  }
}
impl Tape for CursorText {
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
impl CursorText {
  pub fn delete(&mut self) -> bool {
    if self.forward_remainder(1) != 0 ||
      self.text.len() == 0
    {
      false
    } else {
      self.text.remove(self.head);
      true
    }
  }
  pub fn backspace(&mut self) -> bool {
    if self.backward_remainder(1) != 0 {
      false
    } else {
      self.wrap_backward(1);
      self.text.remove(self.head);
      true
    }
  }
  pub fn insert(&mut self, c: char) -> bool {
    if self.head + 1 == self.text.len() || 
      self.text.len() == 0 
    {
      self.text.push(c);
      self.wrap_forward(1);
      true
    } else {
      self.text.insert(self.head, c);
      self.wrap_forward(1);
      true
    }
  }
}

#[derive(Clone)]
pub struct CursorDoc<T> {
  pub head: usize,
  pub text: Vec<T>,
}
impl<T> Default for CursorDoc<T> 
where T: Default
{
  fn default() -> Self {
    Self {head: 0, text: vec![T::default()]}
  }
}
impl<T> Tape for CursorDoc<T> {
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
impl<T> CursorDoc<T> 
where T: Tape
{
  pub fn move_left(&mut self, step: usize) -> bool {
    self.wrap_left(step) == step
  }
  pub fn move_right(&mut self, step: usize) -> bool {
    self.wrap_right(step) == step
  }
  pub fn move_up(&mut self, step: usize) -> bool {
    let x = self.text[self.head].head();
    if self.wrap_backward(step) == 0 {
      self.text[self.head].fit(x);
      true
    } else {
      false
    }
  }
  pub fn move_down(&mut self, step: usize) -> bool {
    let x = self.text[self.head].head();
    if self.wrap_forward(step) == 0 {
      self.text[self.head].fit(x);
      true
    } else {
      false
    }
  }
  pub fn wrap_left(&mut self, step: usize) -> usize {
    let remainder = self
      .text[self.head].wrap_backward(step);
    // no wrapping required
    if remainder == 0 {
      0
    // try going up
    } else if self.wrap_backward(1) == 0 {
      self.text[self.head].move_to_end();
      self.wrap_left(remainder)
    } else {
      remainder
    }
  }
  pub fn wrap_right(&mut self, step: usize) -> usize {
    let remainder = self
      .text[self.head].wrap_forward(step);
    // no wrapping required
    if remainder == 0 {
      0
    // try going down
    } else if self.wrap_forward(1) == 0 {
      self.text[self.head].move_to_start();
      self.wrap_right(remainder)

    } else {
      remainder
    }
  }
}
