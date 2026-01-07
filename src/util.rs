// util

// a rectangle specified by a point and some lengths
#[derive(Clone, Debug)]
pub struct Rect {
    pub x: u16, 
    pub y: u16, 
    pub w: u16, 
    pub h: u16,
}
impl Rect {
    pub fn verticle(&self) -> Result<Range, String> {
        Range::new(usize::from(self.y), usize::from(self.y + self.h))
    }
    pub fn horizontal(&self) -> Result<Range, String> {
        Range::new(usize::from(self.x), usize::from(self.x + self.w))
    }
}
#[derive(Clone, Debug)]
pub struct Range {
    a: usize,
    b: usize,
}
impl Range {
    pub fn new(a: usize, b: usize) -> Result<Self, String> {
        match a > b {
            true => 
                Err(format!("invalid range: (a: {}) > (b: {})", a, b)),
            false => 
                Ok(Self {a: a, b: b}),
        }
    }
    pub fn len(&self) -> usize {
        self.b - self.a
    }
    pub fn start(&self) -> usize {
        self.a
    }
    pub fn end(&self) -> usize {
        self.b
    }
}
#[derive(Clone, Debug)]
pub struct ScrollingCursor {
    inner:     Range,
    outer:     Range,
    buf:       usize,
    cursor:    usize,
    scroll:    usize,
    maxscroll: usize,
}
impl ScrollingCursor {
    // private helper returning (outer, inner) ranges
    fn get_ranges(len: usize, r: &Range, buf: usize) -> (Range, Range) {
        if len < r.len() {
            let range = Range::new(r.start(), r.start() + len).unwrap();
            return (range.clone(), range)
        } else {
            // if buf is too big return input
            let inner = 
                match Range::new(r.start() + buf, r.end() - (buf + 1)) {
                    Ok(range) => range,
                    _         => r.clone(),
                };
            return (r.clone(), inner)
        }
    }
    pub fn new(len: usize, r: &Range, buf: u8) -> Self {
        let buf = usize::from(buf);
        let (outer, inner) = Self::get_ranges(len, r, buf);
        Self {
            buf:       buf,
            scroll:    0, 
            maxscroll: len - outer.len(),
            cursor:    (outer.start() + outer.end() - 1) / 2, 
            outer:     outer,
            inner:     inner,
        }
    }
    pub fn resize(&mut self, len: usize, r: &Range) {
        let (outer, inner) = Self::get_ranges(len, r, self.buf);
        self.outer     = outer;
        self.inner     = inner;
        self.maxscroll = len - self.outer.len();
        self.scroll    = std::cmp::min(self.scroll, self.maxscroll);
        self.cursor    = (self.outer.start() + self.outer.end() - 1) / 2;
    }
    pub fn move_up(&mut self, mut step: usize) -> bool {
        // cursor is bounded only by outer_rng
        if self.scroll == usize::MIN {
            if self.cursor == self.outer.start() {
                return false
            } else if self.outer.start() + step <= self.cursor {
                self.cursor -= step; 
            } else {
                self.cursor = self.outer.start();
            }
        // cursor is bounded by inner_rng
        } else if (self.inner.start() + step) <= self.cursor {
            self.cursor -= step; 
        // cursor must move, then scroll, then maybe move again
        } else {
            // lower step by the amount to move cursor
            step -= self.inner.start() - self.cursor;
            // move cursor
            self.cursor = self.inner.start();
            // the rest of step is accomplished with scroll alone
            if step <= self.scroll {
                self.scroll -= step;
            // must move cursor again
            } else {
                step -= self.scroll;
                self.scroll = usize::MIN;
                // if this fails, step was too large from the start
                if self.outer.start() + step <= self.cursor {
                    self.cursor -= step; 
                } else {
                    self.cursor = self.outer.start();
                }
            }
        }
        return true
    }
    pub fn move_down(&mut self, mut step: usize) -> bool {
        // cursor is bounded only by outer_rng
        if self.scroll == self.maxscroll {
            if self.cursor == self.outer.end() - 1 {
                return false
            } else if self.cursor + step <= self.outer.end() - 1 {
                self.cursor += step;
            } else {
                self.cursor = self.outer.end() - 1;
            }
        // cursor is bounded by inner_rng
        } else if (self.cursor + step) <= self.inner.end() {
            self.cursor += step;
        // cursor must move, then scroll, then it maybe move again
        } else {
            // lower step by the amount to move cursor
            step -= self.inner.end() - self.cursor;
            // move cursor
            self.cursor = self.inner.end();
            // the rest of step is accomplished with scroll alone
            if self.scroll + step <= self.maxscroll {
                self.scroll += step;
            // must move cursor again
            } else {
                step -= self.maxscroll - self.scroll;
                self.scroll = self.maxscroll;
                // if this fails, step was too large from the start
                if self.cursor + step <= self.outer.end() - 1 {
                    self.cursor += step;
                } else {
                    self.cursor = self.outer.end() - 1;
                }
            }
        }
        return true
    }
    // return scroll
    pub fn get_scroll(&self) -> usize {
        self.scroll
    }
    // index of cursor within its range
    pub fn get_index(&self) -> usize {
        self.scroll + (self.cursor - self.outer.start())
    }
    // return u16 for cursor
    pub fn get_cursor(&self) -> u16 {
        u16::try_from(self.cursor).unwrap()
    }
    // return u16 for outer.start
    pub fn get_screen_start(&self) -> u16 {
        u16::try_from(self.outer.start()).unwrap()
    }
    // returns the start and end of displayable text
    pub fn get_display_range(&self) -> (usize, usize) {
        (self.scroll, self.scroll + self.outer.len())
    }
}
// call wrap for each element in the list
pub fn wrap_list(lines: &Vec<String>, w: u16) 
    -> Vec<(usize, String)> 
{
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
                let shortest = match a.len() {
                    0 => b,
                    _ => a,
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
