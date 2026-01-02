// geometry
//
// + resize()
// - view()
// - update()

// a rectangle specified by a point and some lengths
#[derive(Clone, Debug)]
pub struct Rect {
    pub x: u16, 
    pub y: u16, 
    pub w: u16, 
    pub h: u16,
}
impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {x: x, y: y, w: w, h: h}
    }
}
#[derive(Clone, Debug)]
pub struct Cursor {
    cur: u16,
    min: u16,
    max: u16,
}
impl Cursor {
    // sets limits given length of text and rect
    pub fn new(len: usize, rect: &Rect) -> Self {
        let len = match u16::try_from(len) {
            Ok(t) => t, _ => u16::MAX,
        };
        Self {
            min: rect.y,
            cur: rect.y, 
            max: std::cmp::min(len, rect.h),
        }
    }
    pub fn resize(&mut self, len: usize, rect: &Rect) {
        let len = match u16::try_from(len) {
            Ok(t) => t, _ => u16::MAX,
        };
        self.min = rect.y;
        self.max = std::cmp::min(len, rect.h);
        self.cur = (self.min + self.max - 1) / 2;
    }
    pub fn moveup(&mut self, step: u16) -> bool {
        if (self.min + step) <= self.cur {
            self.cur -= step; 
            true
        } else {
            false
        }
    }
    pub fn movedown(&mut self, step: u16) -> bool {
        if (self.cur + step) <= (self.min + self.max - 1) {
            self.cur += step; 
            true 
        } else {
            false
        }
    }
    pub fn getcursor(&self) -> u16 {
        self.cur
    }
    pub fn getmaxcursor(&self) -> u16 {
        self.max
    }
    // index of cursor within its rect
    pub fn getindex(&self) -> usize {
        usize::from(self.cur - self.min)
    }
}
// scroll over data when cursor position is at a limit
// defined by rect
#[derive(Clone, Debug)]
pub struct ScrollingCursor {
    cursor:    Cursor,
    scroll:    usize,
    maxscroll: usize,
}
impl ScrollingCursor {
    // sets limits given length of text and rect
    pub fn new(len: usize, rect: &Rect) -> Self {
        let cursor = Cursor::new(len, rect);
        Self {
            scroll: 0, 
            maxscroll: len - usize::from(cursor.max),
            cursor: cursor,
        }
    }
    // like Self::new but tries to preserve scroll
    pub fn resize(&mut self, len: usize, rect: &Rect) {
        self.cursor.resize(len, rect);
        self.maxscroll = len - usize::from(self.cursor.max);
        self.scroll = std::cmp::min(self.scroll, self.maxscroll);
    }
    // scroll up when cursor is at highest position
    pub fn moveup(&mut self, step: u16) -> bool {
        let scrollstep = usize::from(step);
        if self.cursor.moveup(step) {
            true
        } else if usize::MIN + scrollstep <= self.scroll {
            self.scroll -= scrollstep; 
            true
        } else {
            false
        }
    }
    // scroll down when cursor is at lowest position
    pub fn movedown(&mut self, step: u16) -> bool {
        let scrollstep = usize::from(step);
        if self.cursor.movedown(step) {
            true 
        } else if (self.scroll + scrollstep) <= self.maxscroll {
            self.scroll += scrollstep; 
            true
        } else {
            false
        }
    }
    // index of cursor within its rect
    pub fn getcursor(&self) -> u16 {
        self.cursor.cur
    }
    // index of cursor within its rect
    pub fn getscroll(&self) -> usize {
        self.scroll
    }
    pub fn getscreenstart(&self) -> u16 {
        self.cursor.min
    }
    // index of cursor within its rect
    pub fn getindex(&self) -> usize {
        self.scroll + self.cursor.getindex()
    }
    // returns the start and end of displayable text
    pub fn getdisplayrange(&self) -> (usize, usize) {
        (self.scroll, self.scroll + usize::from(self.cursor.max))
    }
}
