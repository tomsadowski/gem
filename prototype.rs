// common

// call wrap for each element in the list
pub fn wrap_list(lines: &Vec<String>, w: u16) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, l) in lines.iter().enumerate() {
        let v = wrap(l, w);
        for s in v.iter() {
            display.push((i, s.to_string()));
        }
    }
    display
}
// wrap text in terminal
pub fn wrap(line: &str, screenwidth: u16) -> Vec<String> {
    let width = usize::from(screenwidth);
    let length = line.len();
    let mut wrapped: Vec<String> = vec![];
    // assume slice bounds
    let mut start = 0;
    let mut end = width;
    while end < length {
        start = line.ceil_char_boundary(start);
        end = line.floor_char_boundary(end);
        let longest = &line[start..end];
        // try to break line at a space
        match longest.rsplit_once(' ') {
            // there is a space to break on
            Some((a, b)) => {
                let sxtest = match a.len() {
                    0 => b,
                    _ => a,
                };
                wrapped.push(String::from(sxtest.trim()));
                start += sxtest.len();
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
pub fn split_whitespace_once(source: &str) -> (&str, &str) {
    let line = source.trim();
    let (a, b) = {
        if let Some(i) = line.find("\u{0009}") {
            (line[..i].trim(), line[i..].trim())
        } else if let Some(i) = line.find(" ") {
            (line[..i].trim(), line[i..].trim())
        } else {
            (line, line)
        }
    };
    (a, b)
}

#[derive(Clone)]
pub struct ScrollPoint(pub usize, pub usize);
#[derive(Clone)]
pub struct ScreenPoint(pub u16, pub u16);
#[derive(Clone)]
pub struct ScreenSize(pub u16, pub u16);
#[derive(Clone)]
pub struct DataRange(pub usize, pub usize);

#[derive(Clone)]
pub struct ViewColumn {
    pub cursor: u16, 
    pub scroll: usize
}
impl ViewColumn {
    pub fn join_with_x(self, x: ViewColumn) -> View {
        View {
            cursor: ScreenPoint(x.cursor, self.cursor),
            scroll: ScrollPoint(x.scroll, self.scroll),
        }
    }
    pub fn join_with_y(self, y: ViewColumn) -> View {
        View {
            cursor: ScreenPoint(self.cursor, y.cursor),
            scroll: ScrollPoint(self.scroll, y.scroll),
        }
    }
    pub fn move_into(&mut self, bounds: &ViewBound) -> bool {
        let ViewBound(_, bound_type) = bounds;
        let (start, end) = match bound_type {
            BoundType::NoScroll | 
            BoundType::NoSpacer(_) => 
            {
                let ViewBound(ScreenRange(start, end), _) = bounds;
                (start, end)
            } 
            BoundType::Spacer(ScreenRange(start, end), _) =>
            {
                (start, end)
            }
        };
        match (self.cursor >= *start, self.cursor <= *end) {
            (true, true) => {
                false
            }
            (true, false) => {
                self.cursor = *start;
                true
            }
            (_, _) => {
                self.cursor = *end;
                true
            }
        } 
    }
    pub fn move_backward(&mut self, bounds: &ViewBound, step: u16) -> bool {
        let ViewBound(ScreenRange(start, _), bound_type) = bounds;
        let mut step = step;
        match bound_type {
            BoundType::NoScroll => {
                if self.cursor == *start {
                    false
                } else if start + step <= self.cursor {
                    self.cursor -= step;
                    true
                } else {
                    self.cursor = *start;
                    true
                }
            }
            BoundType::NoSpacer(_) => {
                match (self.cursor == *start, self.scroll == usize::MIN) {
                    (true, true) => {
                        false
                    }
                    (false, true) => {
                        if start + step <= self.cursor {
                            self.cursor -= step;
                            true
                        } else {
                            self.cursor = *start;
                            true
                        }
                    }
                    (true, false) => {
                        if usize::from(step) < self.scroll  {
                            self.scroll -= usize::from(step);
                            true
                        } else {
                            self.scroll = usize::MIN;
                            true
                        }
                    }
                    (false, false) => {
                        if start + step <= self.cursor {
                            self.cursor -= step;
                            true
                        } else {
                            step -= self.cursor - start;
                            self.cursor = *start;
                            self.move_backward(bounds, step);
                            true
                        }
                    }
                }
            }
            BoundType::Spacer(ScreenRange(scroll_at, _), _) => {
                match (self.cursor == *scroll_at, self.scroll == usize::MIN) {
                    (_, true) => {
                        if start + step <= self.cursor {
                            self.cursor -= step;
                            true
                        } else {
                            self.cursor = *start;
                            true
                        }
                    }
                    (true, false) => {
                        if usize::from(step) < self.scroll  {
                            self.scroll -= usize::from(step);
                            true
                        } else {
                            step -= u16::try_from(self.scroll)
                                .unwrap_or(u16::MIN);
                            self.scroll = usize::MIN;
                            self.move_backward(bounds, step);
                            true
                        }
                    }
                    (false, false) => {
                        if scroll_at + step <= self.cursor {
                            self.cursor -= step;
                            true
                        } else {
                            step -= self.cursor - scroll_at;
                            self.cursor = *scroll_at;
                            self.move_backward(bounds, step);
                            true
                        }
                    }
                }
            }
        }
    }
    pub fn move_forward(&mut self, bounds: &ViewBound, step: u16)  -> bool {
        let ViewBound(ScreenRange(_, end), bound_type) = bounds;
        let mut step = step;

        match bound_type {
            BoundType::NoScroll => {
                if self.cursor == *end {
                    false
                } else if end + step <= self.cursor {
                    self.cursor -= step;
                    true
                } else {
                    self.cursor = *end;
                    true
                }
            }
            BoundType::NoSpacer(_) => {
                match ( self.cursor == *end, 
                        self.scroll == bounds.get_max_scroll()) 
                {
                    (true, true) => {
                        false
                    }
                    (false, true) => {
                        if self.cursor + step <= *end {
                            self.cursor += step;
                            true
                        } else {
                            self.cursor = *end;
                            true
                        }
                    }
                    (true, false) => {
                        if self.scroll + usize::from(step) < 
                            bounds.get_max_scroll()  
                        {
                            self.scroll += usize::from(step);
                            true
                        } else {
                            self.scroll = bounds.get_max_scroll();
                            true
                        }
                    }
                    (false, false) => {
                        if self.cursor + step <= *end {
                            self.cursor += step;
                            true
                        } else {
                            step -= end - self.cursor;
                            self.cursor = *end;
                            self.move_forward(bounds, step);
                            true
                        }
                    }
                }
            }
            BoundType::Spacer(ScreenRange(_, scroll_at), _) => {
                match ( self.cursor <= *scroll_at, 
                        self.scroll == bounds.get_max_scroll()) 
                {
                    (_, true) => {
                        if self.cursor == *end {
                            false
                        } else if self.cursor + step <= *end {
                            self.cursor += step;
                            true
                        } else {
                            self.cursor = *end;
                            true
                        }
                    }
                    (true, false) => {
                        if self.scroll + usize::from(step) < 
                            bounds.get_max_scroll() 
                        {
                            self.scroll += usize::from(step);
                            true
                        } else {
                            step -= u16::try_from(
                                    bounds.get_max_scroll() - self.scroll)
                                .unwrap_or(u16::MIN);
                            self.scroll = bounds.get_max_scroll();
                            self.move_forward(bounds, step);
                            true
                        }
                    }
                    (false, false) => {
                        if self.cursor + step <= *scroll_at {
                            self.cursor += step;
                            true
                        } else {
                            step += scroll_at - self.cursor;
                            self.cursor = *scroll_at;
                            self.move_forward(bounds, step);
                            true
                        }
                    }
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct Screen {
    pub point: ScreenPoint, 
    pub size:  ScreenSize,
}
impl Screen {
    pub fn get_x(&self) -> ScreenRange {
        let ScreenPoint(x, _) = self.point;
        let ScreenSize(w, _)  = self.size;
        ScreenRange(x, x + w)
    }
    pub fn get_y(&self) -> ScreenRange {
        let ScreenPoint(_, y) = self.point;
        let ScreenSize(_, h)  = self.size;
        ScreenRange(y, y + h)
    }
}
#[derive(Clone)]
pub struct View {
    pub cursor: ScreenPoint,
    pub scroll: ScrollPoint,
}
impl View {
    pub fn get_x(&self) -> ViewColumn {
        let ScreenPoint(cursor, _) = self.cursor;
        let ScrollPoint(scroll, _) = self.scroll;
        ViewColumn {cursor, scroll}
    }
    pub fn get_y(&self) -> ViewColumn {
        let ScreenPoint(_, cursor) = self.cursor;
        let ScrollPoint(_, scroll) = self.scroll;
        ViewColumn {cursor, scroll}
    }
}
#[derive(Clone)]
pub struct ScreenRange(pub u16, pub u16);
impl ScreenRange {
    pub fn from_length(start: u16, len: usize) -> ScreenRange {
        let len = u16::try_from(len).unwrap_or(u16::MIN);
        ScreenRange(start, start + len)
    }
    // if for some reason a > b, just swap them
    pub fn new(a: u16, b: u16) -> ScreenRange {
        match a > b {
            true =>  ScreenRange(b, a),
            false => ScreenRange(a, b),
        }
    }
    pub fn to_data_range(&self) -> DataRange {
        let ScreenRange(a, b) = *self;
        DataRange(usize::from(a), usize::from(b))
    }
    // index of cursor within its range
    pub fn get_index(&self, col: &ViewColumn) -> usize {
        col.scroll + usize::from(col.cursor - self.0)
    }
    pub fn length(&self) -> usize {
        usize::from(self.1 - self.0)
    }
}
pub enum BoundType {
    NoScroll,
    NoSpacer(usize),
    Spacer(ScreenRange, usize)
}
pub struct ViewBound(ScreenRange, BoundType);
impl ViewBound {
    pub fn new(screen_range: ScreenRange, spacer: u16, len: usize) 
        -> ViewBound 
    {
        let ScreenRange(start, end) = screen_range;
        if screen_range.length() >= len {
            ViewBound(
                ScreenRange::from_length(start, len), 
                BoundType::NoScroll)
        } else {
            if usize::from(spacer) * 2 >= screen_range.length() {
                ViewBound(
                    screen_range,
                    BoundType::NoSpacer(len))
            } else {
                let scroll_a        = start + spacer;
                let scroll_b        = end - spacer - 1;
                let scroll_points   = ScreenRange::new(scroll_a, scroll_b);
                ViewBound(
                    screen_range, 
                    BoundType::Spacer(scroll_points, len))
            }
        }
    }
    pub fn get_max_scroll(&self) -> usize {
        let ViewBound(range, bound_type) = self;
        match bound_type {
            BoundType::NoScroll => {
                0
            }
            BoundType::NoSpacer(l) | BoundType::Spacer(_, l) => {
                l - range.length()
            }
        }
    }
    // returns the start and end of displayable text
    pub fn get_data_range(&self, col: ViewColumn) -> DataRange {
        let ViewBound(range, bound_type) = self;
        match bound_type {
            BoundType::NoScroll => {
                range.to_data_range()
            }
            BoundType::NoSpacer(_) | BoundType::Spacer(_, _) => {
                DataRange(col.scroll, col.scroll + range.length())
            }
        }
    }
}
//  pub struct Bounds {
//      pub x: ScrollBound,
//      pub y: ScrollBound,
//  }
//  impl Bounds {
//      // if spacer is too large, scroll_points == screen_range
//      pub fn new(screen: Screen, hspace: u16, vspace: u16) -> ScrollBounds {
//          Self {
//              x:  ScrollBound::new(screen.get_x(), hspace), 
//              y:  ScrollBound::new(screen.get_y(), vspace)
//          }
//      }
//      pub fn move_left(&self, view: &View, step: u16) -> Option<View> {
//          view.get_x()
//              .move_backward(&self.x, step)
//              .map(|v| v.join_with_y(view.get_y()))
//      }
//      pub fn move_right(&self, view: &View, len: usize, step: u16) -> Option<View> {
//          view.get_x()
//              .move_forward(&self.x, len, step)
//              .map(|v| v.join_with_y(view.get_y()))
//      }
//      pub fn move_up(&self, view: &View, step: u16) -> Option<View> {
//          view.get_y()
//              .move_backward(&self.y, step)
//              .map(|v| v.join_with_x(view.get_x()))
//      }
//      pub fn move_down(&self, view: &View, len: usize, step: u16) -> Option<View> {
//          view.get_y()
//              .move_forward(&self.y, len, step)
//              .map(|v| v.join_with_x(view.get_x()))
//      }
//      pub fn get_x_data_range(&self, len: usize, view: View) -> DataRange {
//          let ViewColumn(_, scroll) = view.get_x();
//          let end = std::cmp::min(scroll + self.x.screen_range.length(), len);
//          DataRange(scroll, end)
//      }
//      pub fn get_y_data_range(&self, len: usize, view: View) -> DataRange {
//          let ViewColumn(_, scroll) = view.get_y();
//          let end = std::cmp::min(scroll + self.y.screen_range.length(), len);
//          DataRange(scroll, end)
//      }
//  }
