// gem/src/widget
// backend agnostic

use crossterm::{
    QueueableCommand, cursor, style,
    event::{KeyCode},
    style::{Colors},
};
use std::{
    io::{self, Stdout},
    cmp::{min},
};

// associates implementor with a color
pub trait GetColors {
    fn getcolors(&self) -> Colors;
}
// a rectangle specified by a point and some lengths
#[derive(Clone, Debug)]
pub struct Rect {
    pub x: u16, pub y: u16, pub w: u16, pub h: u16,
}
impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {x: x, y: y, w: w, h: h}
    }
}
// scroll over data when cursor position is at a limit
// defined by rect
#[derive(Clone, Debug)]
pub struct ScrollingCursor {
    pub scroll: usize,
    pub maxscroll: usize,
    pub cursor: u16,
    pub rect: Rect,
}
impl ScrollingCursor {
    // sets limits given length of text and rect
    pub fn new(textlength: usize, rect: &Rect) -> Self {
        let len = match u16::try_from(textlength) {
            Ok(t) => t, _ => u16::MAX,
        };
        match len < rect.h {
            // no scrolling allowed
            true => Self {
                rect: Rect::new(rect.x, rect.y, rect.w, len),
                cursor: rect.y, 
                scroll: 0, 
                maxscroll: 0,
            },
            // scrolling allowed
            false => Self {
                rect: rect.clone(),
                cursor: rect.y, 
                scroll: 0, 
                maxscroll: textlength - usize::from(rect.h),
            },
        }
    }
    // like Self::new but tries to preserve scroll
    pub fn resize(&mut self, textlength: usize, rect: &Rect) {
        let len = match u16::try_from(textlength) {
            Ok(t) => t, _ => u16::MAX,
        };
        match len < rect.h {
            // no scrolling allowed
            true => {
                self.rect = Rect::new(rect.x, rect.y, rect.w, len);
                self.scroll = 0;
                self.maxscroll = 0;
            },
            // scrolling allowed
            false => {
                self.rect = rect.clone();
                self.scroll = min(self.scroll, self.maxscroll);
                self.maxscroll = textlength - usize::from(rect.h);
            },
        }
        self.cursor = (self.rect.y + self.rect.h - 1) / 2;
    }
    // scroll up when cursor is at highest position
    pub fn moveup(&mut self, step: u16) -> bool {
        let scrollstep = usize::from(step);
        if (self.rect.y + step) <= self.cursor {
            self.cursor -= step;
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
        if (self.cursor + step) <= (self.rect.y + self.rect.h - 1) {
            self.cursor += step;
            true 
        } else if (self.scroll + scrollstep) <= self.maxscroll {
            self.scroll += scrollstep;
            true
        } else {
            false
        }
    }
    // returns the start and end of displayable text
    pub fn slicebounds(&self) -> (usize, usize) {
        let start = self.scroll;
        let end = self.scroll + usize::from(self.rect.h);
        (start, end)
    }
    // index of cursor within its rect
    pub fn index(&self) -> usize {
        usize::from(self.cursor - self.rect.y)
    }
}
// wrap text in terminal
pub fn wrap(line: &str, screenwidth: u16) -> Vec<String> {
    let width = usize::from(screenwidth);
    let length = line.len();
    let mut wrapped: Vec<String> = vec![];
    // assume slice bounds
    let (mut start, mut end) = (0, width);
    while end < length {
        let longest = &line[start..end];
        // try to break line at a space
        match longest.rsplit_once(' ') {
            // there is a space to break on
            Some((a, b)) => {
                let shortest = match a.len() {
                    0 => b,
                    _ => a,
                };
                wrapped.push(String::from(shortest));
                start += shortest.len();
                end = start + width;
            }
            // there is no space to break on
            None => {
                wrapped.push(String::from(longest));
                start = end;
                end += width;
            }
        }
    }
    // add the remaining text
    if start < length {
        wrapped.push(String::from(&line[start..length]));
    }
    wrapped
}
// cut text in terminal, adding "..." to indicate that it 
// continues beyond the screen
pub fn cut(line: &str, screenwidth: u16) -> String {
    let mut width = usize::from(screenwidth);
    if line.len() < width {
        return String::from(line)
    } else {
        width -= 2;
        let longest = &line[..width];
        match longest.rsplit_once(' ') {
            Some((a, b)) => {
                let shortest = match a.len() {
                    0 => b,
                    _ => a,
                };
                return format!("{}..", shortest)
            }
            None => {
                return format!("{}..", longest)
            }
        }

    }
}
// call cut for each element in the list
pub fn cutlist<T>(lines: &Vec<(T, String)>, w: u16) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, (_, l)) in lines.iter().enumerate() {
        display.push((i, cut(l, w)));
    }
    display
}
// call wrap for each element in the list
pub fn wraplist<T>(lines: &Vec<(T, String)>, w: u16) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, (_, l)) in lines.iter().enumerate() {
        let v = wrap(l, w);
        for s in v.iter() {
            display.push((i, s.to_string()));
        }
    }
    display
}
#[derive(Clone, Debug)]
pub enum DialogMsg {
    None,
    Cancel,
    Submit,
}
#[derive(Clone, Debug)]
pub enum InputType {
    None,
    Choose((char, Vec<(char, String)>)),
    Input(String),
}
#[derive(Clone, Debug)]
pub struct Dialog<T> {
    rect: Rect,
    prompt: String,
    pub action: T,
    pub input: InputType,
}
impl<T: Clone + std::fmt::Debug> Dialog<T> {
    pub fn new(rect: &Rect, action: T, input: InputType, prompt: &str) -> Self 
    {
        Self {
            rect: rect.clone(),
            action: action,
            input: input,
            prompt: String::from(prompt), 
        }
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        stdout
            .queue(cursor::MoveTo(self.rect.x + 2, self.rect.y + 2))?
            .queue(style::Print(self.prompt.as_str()))?
            .queue(cursor::MoveTo(self.rect.x + 2, self.rect.y + 4))?
            .queue(style::Print(format!("{:?}", self.input)))?;
        Ok(())
    }
    // No wrapping yet, so resize is straightforward
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
    }
    // Keycode has various meanings depending on the InputType.
    // The match statement might be moved to impl InputType
    pub fn update(&mut self, keycode: &KeyCode) -> Option<DialogMsg> {
        match (&mut self.input, keycode) {
            // Pressing Escape always cancels
            (_, KeyCode::Esc) => {
                Some(DialogMsg::Cancel)
            }
            // Pressing Enter in a choosebox means nothing
            (InputType::Choose(_), KeyCode::Enter) => {
                Some(DialogMsg::None)
            }
            // Otherwise, pressing Enter means Submit
            (_, KeyCode::Enter) => {
                Some(DialogMsg::Submit)
            }
            // Backspace works in inputbox
            (InputType::Input(v), KeyCode::Backspace) => {
                v.pop();
                Some(DialogMsg::None)
            }
            // Typing works in inputbox
            (InputType::Input(v), KeyCode::Char(c)) => {
                v.push(*c);
                Some(DialogMsg::None)
            }
            // Check for meaning in choosebox
            (InputType::Choose(t), KeyCode::Char(c)) => {
                let chars: Vec<char> = t.1.iter().map(|e| e.0).collect();
                match chars.contains(&c) {
                    true => {
                        t.0 = *c;
                        Some(DialogMsg::Submit)
                    }
                    false => None,
                }
            }
            _ => None,
        }
    }
}
// enables the selection of metadata (T) behind formatted text.
#[derive(Clone, Debug)]
pub struct Selector<T> {
    rect: Rect,
    wrap: bool,
    pub cursor: ScrollingCursor,
    source: Vec<(T, String)>,
    display: Vec<(usize, String)>,
} 
impl<T: Clone + GetColors> Selector<T> {
    pub fn new(rect: &Rect, source: Vec<(T, String)>, wrap: bool) -> Self {
        let display = match wrap {
            true => wraplist(&source, rect.w),
            false => cutlist(&source, rect.w),
        };
        return Self {
            rect: rect.clone(),
            wrap: wrap,
            cursor: ScrollingCursor::new(display.len(), &rect),
            source: source,
            display: display,
        }
    }
    pub fn resize(&mut self, rect: &Rect) {
        self.rect = rect.clone();
        self.display = match self.wrap {
            true => wraplist(&self.source, rect.w),
            false => cutlist(&self.source, rect.w),
        };
        self.cursor.resize(self.display.len(), rect);
    }
    pub fn view(&self, mut stdout: &Stdout) -> io::Result<()> {
        let (a, b) = self.cursor.slicebounds();
        for (j, (i, text)) in self.display[a..b].iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(self.rect.x, self.rect.y + j as u16))?
                .queue(style::SetColors(self.source[*i].0.getcolors()))?
                .queue(style::Print(text.as_str()))?;
        }
        stdout.queue(cursor::MoveTo(0, self.cursor.cursor))?;
        Ok(())
    }
    pub fn selectundercursor(&self) -> &T {
        &self.source[self.display[self.cursor.index()].0].0
    }
} 
