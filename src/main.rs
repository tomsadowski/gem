#![allow(dead_code)]
#![allow(unused_imports)]
use crossterm::{
    QueueableCommand,
    terminal::{self, Clear, ClearType},
    cursor::{self, MoveTo},
    style::{self, Color, SetForegroundColor, SetBackgroundColor, Print},
    event::{self, Event, KeyEvent, KeyEventKind, KeyCode, KeyModifiers},};
use std::{
    time::{Duration}, 
    io::{self, stdout, Stdout, Write, Read},
    net::{TcpStream, ToSocketAddrs},
    cmp::min,
    fs,};
use url::{Url, ParseError};
use serde::Deserialize;
use native_tls::TlsConnector;

pub fn u16_or_0(u: usize) -> u16 {
    u16::try_from(u).unwrap_or(u16::MIN)
}
#[derive(Clone, Debug)]
pub struct Range16 {
    pub start:  u16, 
    pub end:    u16
}
impl Range16 {
    // if for some reason a > b, just swap them
    pub fn new(start: u16, end: u16) -> Range16 {
        if start > end {
            Range16 {start: end, end: start}
        } else {
            Range16 {start: start, end: end}
        }
    }
    pub fn get_data_end(&self, dlen: usize) -> u16 {
        let data_end = usize::from(self.start) + dlen.saturating_sub(1);
        let scr_end  = usize::from(self.end).saturating_sub(1);
        u16_or_0(min(data_end, scr_end))
    }
    pub fn get_max_scroll(&self, dlen: usize) -> usize {
        dlen.saturating_sub(self.len())
    }
    pub fn contains(&self, n: u16) -> bool {
        self.start <= n && n <= self.end
    }
    pub fn len16(&self) -> u16 {
        self.end.saturating_sub(self.start)
    }
    pub fn len(&self) -> usize {
        usize::from(self.len16())
    }
}
#[derive(Clone, Debug)]
pub struct DataScreenRange {
    pub inner: Range16,
    pub outer: Range16,
}
impl DataScreenRange {
    pub fn get_data_end(&self, dlen: usize) -> u16 {
        self.outer.get_data_end(dlen)
    }
    pub fn get_max_scroll(&self, dlen: usize) -> usize {
        self.outer.get_max_scroll(dlen)
    }
}
pub struct DataScreen {
    pub inner: Rect,
    pub outer: Screen,
}
impl DataScreen {
    pub fn new(outer: &Screen, x: u16, y: u16) -> DataScreen {
        Self {
            inner: outer.r.crop_x(x).crop_y(y),
            outer: outer.clone(),
        }
    }
    pub fn get_x_rng(&self) -> DataScreenRange {
        DataScreenRange {
            inner: self.inner.x(), 
            outer: self.outer.r.x()
        }
    }
    pub fn get_y_rng(&self) -> DataScreenRange {
        DataScreenRange {
            inner: self.inner.y(), 
            outer: self.outer.r.y()
        }
    }
}
#[derive(Clone, Debug)]
pub struct PosCol {pub cursor: u16, pub scroll: usize}
impl PosCol {
    pub fn origin(rng: &Range16) -> PosCol {
        PosCol {cursor: rng.start, scroll: 0}
    }
    // index of cursor within its range
    pub fn data_idx_cap(&self, rng: &Range16, max: usize) -> usize {
        let idx = if self.cursor > rng.start {
            let p = self.cursor.saturating_sub(rng.start);
            self.scroll + usize::from(p)
        } else {self.scroll};
        min(idx, max)
    }
    // index of cursor within its range
    pub fn data_idx(&self, rng: &Range16) -> usize {
        if self.cursor > rng.start {
            let p = self.cursor.saturating_sub(rng.start);
            self.scroll + usize::from(p)
        } else {self.scroll}
    }
    // returns the start and end of displayable text
    pub fn data_range(&self, rng: &Range16, len: usize) -> (usize, usize) {
        if len < rng.len() {(0, len)} 
        else {(self.scroll, min(self.scroll + rng.len(), len))}
    }
    // called when screen resizes
    pub fn fit(rng: &Range16, idx: usize, len: usize) -> PosCol {
        let rng_len = rng.len();
        let max_scroll = len.saturating_sub(rng_len);
        if idx < rng_len {
            let cursor = rng.start + u16_or_0(idx);
            PosCol {cursor, scroll: 0}
        } else if idx > max_scroll {
            let cursor = rng.start + 
                u16_or_0(idx.saturating_sub(max_scroll));
            PosCol {
                cursor: cursor, 
                scroll: max_scroll
            }
        } else {
            let scroll = idx.saturating_sub(rng_len / 2);
            let cursor = rng.start + 
                u16_or_0(idx.saturating_sub(scroll));
            PosCol {
                cursor: cursor, 
                scroll: scroll
            }
        }
    }
    pub fn move_into(&mut self, dscr: &DataScreenRange, len: usize) {
        let (start, end) = 
            if len < dscr.outer.len() {
                self.scroll = 0;
                let len = u16_or_0(len);
                (dscr.outer.start, dscr.outer.start + len)
            } else {
                (dscr.outer.start, dscr.inner.end)
            };
        if self.cursor < start {
            self.cursor = start;
        }
        else if self.cursor >= end {
            self.cursor = end;
        }
    }
    pub fn move_backward(   &mut self, 
                            dscr: &DataScreenRange, 
                            mut step: u16) -> bool
    {
        match (self.cursor == dscr.outer.start, self.scroll == usize::MIN) {
            // nowhere to go, nothing to change
            (true, true) => {
                return false
            }
            // move scroll
            (true, false) => {
                if usize::from(step) < self.scroll  {
                    self.scroll -= usize::from(step);
                } else {
                    self.scroll = usize::MIN;
                }
            }
            // move cursor
            (false, true) => {
                if dscr.outer.start + step <= self.cursor {
                    self.cursor -= step;
                } else {
                    self.cursor = dscr.outer.start;
                }
            }
            // move cursor and maybe scroll
            (false, false) => {
                if dscr.inner.start + step <= self.cursor {
                    self.cursor -= step;
                } else if dscr.inner.start == self.cursor {
                    if usize::from(step) <= self.scroll {
                        self.scroll -= usize::from(step);
                    } else {
                        step -= u16_or_0(self.scroll);
                        self.scroll = usize::MIN;
                        self.move_backward(dscr, step);
                    }
                } else {
                    step -= self.cursor.saturating_sub(dscr.inner.start);
                    self.cursor = dscr.inner.start;
                    self.move_backward(dscr, step);
                }
            }
        }
        return true
    }
    pub fn move_forward(    &mut self,
                            dscr: &DataScreenRange, 
                            dlen: usize,
                            mut step: u16 ) -> bool
    {
        let screen_data_end = dscr.get_data_end(dlen);
        let max_scroll      = dscr.get_max_scroll(dlen);
        match (self.cursor == screen_data_end, self.scroll == max_scroll) {
            // nowhere to go, nothing to change
            (true, true) => {
                return false
            }
            // move scroll
            (true, false) => {
                if self.scroll + usize::from(step) >= max_scroll {
                    self.scroll += usize::from(step);
                } else {
                    self.scroll = max_scroll;
                }
            }
            // move cursor
            (false, true) => {
                if self.cursor + step <= screen_data_end {
                    self.cursor += step;
                } else {
                    self.cursor = screen_data_end;
                }
            }
            (false, false) => {
                if self.cursor + step <= dscr.inner.end {
                    self.cursor += step;
                } else if self.cursor == dscr.inner.end {
                    if self.scroll + usize::from(step) <= max_scroll {
                        self.scroll += usize::from(step);
                    } else {
                        let diff = 
                            u16_or_0(max_scroll.saturating_sub(self.scroll));
                        step = step.saturating_sub(diff);
                        self.scroll = max_scroll;
                        self.move_forward(dscr, dlen, step);
                    }
                } else {
                    let diff = dscr.inner.end.saturating_sub(self.cursor);
                    step = step.saturating_sub(diff);
                    self.cursor = dscr.inner.end;
                    self.move_forward(dscr, dlen, step);
                }
            }
        }
        return true
    }
}
#[derive(Clone, Debug)]
pub struct Pos {
    pub x: PosCol,
    pub y: PosCol, 
}
impl Pos {
    pub fn origin(rect: &Rect) -> Pos {
        Pos {
            x: PosCol::origin(&rect.x()),
            y: PosCol::origin(&rect.y())}
    }
    pub fn move_left(&mut self, dscr: &DataScreen, step: u16) -> bool {
        self.x.move_backward(&dscr.get_x_rng(), step)
    }
    pub fn move_right(  &mut self,
                        dscr: &DataScreen, 
                        data: &Vec<usize>,
                        step: u16 ) -> bool
    {
        let x_len = {
            let idx = self.y.data_idx(&dscr.outer.r.y());
            if idx >= data.len() {0} 
            else {data[idx]}
        };
        self.x.move_forward(&dscr.get_x_rng(), x_len, step)
    }
    pub fn move_up( &mut self,
                    dscr: &DataScreen, 
                    data: &Vec<usize>,
                    step: u16 ) -> bool
    {
        if self.y.move_backward(&dscr.get_y_rng(), step) {
            self.move_into_x(dscr, data); true
        } else {false}
    }
    pub fn move_down(   &mut self,
                        dscr: &DataScreen, 
                        data: &Vec<usize>,
                        step: u16 ) -> bool
    {
        if self.y.move_forward(&dscr.get_y_rng(), data.len(), step) {
            self.move_into_x(dscr, data); true
        } else {false}
    }
    pub fn move_into_x(&mut self, dscr: &DataScreen, data: &Vec<usize>) {
        let idx = {
            let idx1 = self.y.data_idx(&dscr.outer.r.y());
            let idx2 = data.len().saturating_sub(1);
            min(idx1, idx2)
        };
        self.x.move_into(&dscr.get_x_rng(), data[idx]);
    }
    pub fn move_into_y(&mut self, dscr: &DataScreen, data: &Vec<usize>) {
        self.y.move_into(&dscr.get_y_rng(), data.len());
    }
    pub fn get_ranges(&self, dscr: &DataScreen, data: &Vec<usize>) 
        -> Vec<(u16, usize, usize, usize)>
    {
        let x_rng = dscr.outer.r.x();
        let y_rng = dscr.outer.r.y();
        let mut vec: Vec<(u16, usize, usize, usize)> = vec![];
        let (start, end) = self.y.data_range(&y_rng, data.len());
        for (e, i) in (start..end).into_iter().enumerate() {
            let (a, b)  = self.x.data_range(&x_rng, data[i]);
            let scr_idx = y_rng.start + (e as u16);
            vec.push((scr_idx, i, a, b));
        }
        vec
    }
}
#[derive(Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: usize,
    pub h: usize,
}
impl Rect {
    pub fn new(w: usize, h: usize) -> Self {
        Self {x: 0, y: 0, w: w, h: h}
    }
    pub fn x(&self) -> Range16 {
        Range16 {
            start: self.x, end: self.x + u16_or_0(self.w)
        }
    }
    pub fn y(&self) -> Range16 {
        Range16 {
            start: self.y, end: self.y + u16_or_0(self.h)
        }
    }
    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w; self.h = h;
    }
    pub fn crop_y(&self, step: u16) -> Self {
        let rect = self.clone();
        rect.crop_north(step).crop_south(step)
    }
    pub fn crop_x(&self, step: u16) -> Self {
        let rect = self.clone();
        rect.crop_east(step).crop_west(step)
    }
    pub fn crop_south(&self, step: u16) -> Self {
        let mut rect = self.clone();
        if usize::from(step) < rect.h {
            rect.h -= usize::from(step);
        }
        rect
    }
    pub fn crop_east(&self, step: u16) -> Self {
        let mut rect = self.clone();
        if usize::from(step) < rect.w {
            rect.w -= usize::from(step)
        }
        rect
    }
    pub fn crop_north(&self, step: u16) -> Self {
        let mut rect = self.clone();
        if step * 2 < rect.y {
            rect.y += step;
            rect.h -= usize::from(step);
        }
        rect
    }
    pub fn crop_west(&self, step: u16) -> Self {
        let mut rect = self.clone();
        if usize::from(step) * 2 < rect.w.saturating_sub(usize::from(rect.x)) {
            rect.x += step;
            rect.w -= usize::from(step);
        }
        rect
    }
}
#[derive(Clone)]
pub struct Screen {
    pub r: Rect, 
    pub j: usize, 
    pub b: Vec<Vec<u8>>
} impl Screen {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            b: vec![vec![u8::MIN; w]; h], 
            r: Rect::new(w, h), 
            j: 0
        }
    }
    pub fn resize(&mut self, w: usize, h: usize) {
        self.b = vec![vec![u8::MIN; w]; h];
        self.r.resize(w, h);
    }
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        let mut y = self.r.y;
        writer.queue(MoveTo(self.r.x, y))?;
        for row in &self.b {
            if let Ok(c) = std::str::from_utf8(&row) {
                writer.queue(Print(c))?
                    .queue(MoveTo(self.r.x, y))?;
                y += 1;
            }
        }
        writer.flush()
    }
}
pub struct DisplayText {
    pub color: Color,
    pub text:  String,
    pub wrap:  bool,
}
impl DisplayText {
    pub fn new(text: &str, color: Color, wrap: bool) -> Self {
        Self {text: text.into(), color, wrap}
    }
}
pub struct DisplayDoc {
    pub src: Vec<DisplayText>,
    pub txt: Vec<(usize, String)>,
    pub i: usize,
    pub j: usize,
} 
impl DisplayDoc {
    pub fn new(src: Vec<DisplayText>, w: usize) -> Self {
        let txt = wrap_list(&src, w);
        Self {txt: txt, src: src, i: 0, j: 0}
    }
    pub fn rewrap(&mut self, width: usize) {
        self.txt = wrap_list(&self.src, width);
    }
    pub fn render(&self, scr: &mut Screen) -> io::Result<()> {
        for ((idx, txt), line) in (&self.txt[self.i..]).iter().zip(&mut scr.b) {
            line.queue(SetForegroundColor(self.src[*idx].color))?
                .queue(Print(txt))?;
        }
        Ok(())
    }
} 
pub fn split_whitespace_once(source: &str) -> (&str, &str) {
    let line = source.trim();
    if let Some(i) = line.find("\u{0009}") {
        (line[..i].trim(), line[i..].trim())
    } else if let Some(i) = line.find(" ") {
        (line[..i].trim(), line[i..].trim())
    } else {(line, line)}
}
pub fn wrap_list(lines: &Vec<DisplayText>, w: usize) -> Vec<(usize, String)> {
    let mut display: Vec<(usize, String)> = vec![];
    for (i, l) in lines.iter().enumerate() {
        let v = if l.wrap {wrap(&l.text, w)} else {vec![l.text.clone()]};
        for s in v.iter() {display.push((i, s.to_string()));}
    }
    display
}
pub fn wrap(line: &str, width: usize) -> Vec<String> {
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
                let shortest = match a.len() {0 => b, _ => a};
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
#[derive(Clone, PartialEq, Debug)]
pub enum Scheme {Gemini, Gopher, Http, Unknown}
pub fn parse_scheme(url: &Url) -> Scheme {
    match url.scheme() {
        "gemini" => Scheme::Gemini,
        "gopher" => Scheme::Gopher,
        "http"   => Scheme::Http,
        "https"  => Scheme::Http,
        _        => Scheme::Unknown,
    }
}
pub fn join_if_relative(base: &Url, url_str: &str) -> Result<Url, ParseError> {
    Url::parse(url_str).or_else(|e|
        if let ParseError::RelativeUrlWithoutBase = e {
            base.join(url_str)
        } else {Err(e)}
    )
}
pub struct GemDoc {
    pub url:    Url,
    pub status: Status,
    pub msg:    String,
    pub doc:    Vec<(GemType, String)>,
}
impl GemDoc {
    pub fn new(url: &Url) -> Result<Self, String> {
        let (response, content) = get_data(url)
            .map_err(|e| e.to_string())?;
        let (status, msg) = parse_status(&response)
            .map_err(|e| e.to_string())?;
        let doc = match status {
            Status::Success => parse_doc(&content, url),
            _ => {
                let msg = 
                    format!("rspns: stts: {:?}, msg: {}", status, msg);
                vec![(GemType::Text, msg)]
            }
        };
        let gem_doc = Self {
            url:    url.clone(),
            status: status,
            msg:    msg,
            doc:    doc,
        };
        Ok(gem_doc)
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum GemType {
    HeadingOne,
    HeadingTwo,
    HeadingThree,
    Text, 
    PreFormat,
    Link(Scheme, Url),
    BadLink(String),
    ListItem,
    Quote,
} 
pub fn parse_doc(text_str: &str, source: &Url) -> Vec<(GemType, String)> {
    let mut vec = vec![];
    let mut preformat = false;
    for line in text_str.lines() {
        if let Some(("```", _)) = line.split_at_checked(3) {
            preformat = !preformat;
        } else if preformat {
            vec.push((GemType::PreFormat, line.into()));
        } else {
            let (gem, text) = parse_formatted(line, source);
            vec.push((gem, text.into()));
        }
    }
    vec
}
fn parse_formatted(line: &str, source: &Url) -> (GemType, String) {
    // look for 3 character symbols
    if let Some(("###", text)) = line.split_at_checked(3) {
        return (GemType::HeadingThree, text.into())
    }
    // look for 2 character symbols
    if let Some((symbol, text)) = line.split_at_checked(2) {
        if symbol == "=>" {
            let (url_str, link_str) = split_whitespace_once(text);
            match join_if_relative(source, url_str) {
                Ok(url) =>
                    return (
                        GemType::Link(parse_scheme(&url), url), 
                        link_str.into()),
                Err(s) => 
                    return (GemType::BadLink(s.to_string()), link_str.into())
            }
        } else if symbol == "##" {
            return (GemType::HeadingTwo, text.into())
        }
    }
    // look for 1 character symbols
    if let Some((symbol, text)) = line.split_at_checked(1) {
        if symbol == ">" {
            return (GemType::Quote, text.into())
        } else if symbol == "*" {
            return (GemType::ListItem, format!("- {}", text))
        } else if symbol == "#" {
            return (GemType::HeadingOne, text.into())
        }
    }
    return (GemType::Text, line.into())
}
#[derive(Debug, Clone)]
pub enum Status {
    InputExpected,
    InputExpectedSensitive,
    Success,
    RedirectTemporary,
    RedirectPermanent,
    FailTemporary,
    FailServerUnavailable,
    FailCGIError,
    FailProxyError,
    FailSlowDown,
    FailPermanent,
    FailNotFound,             
    FailGone,                 
    FailProxyRequestRefused,  
    FailBadRequest,           
    CertRequiredClient,
    CertRequiredTransient,   
    CertRequiredAuthorized,  
    CertNotAccepted,         
    FutureCertRejected,      
    ExpiredCertRejected,     
}
pub fn parse_status(line: &str) -> Result<(Status, String), String> {
    let (code_str, msg) = split_whitespace_once(line);
    let code = code_str.parse::<u8>().map_err(|e| e.to_string())?;
    let status = get_status(code)?;
    Ok((status, msg.into()))
}
fn get_status(code: u8) -> Result<Status, String> {
    match code {
        10 | 12..=19 => Ok(Status::InputExpected),
        11 =>           Ok(Status::InputExpectedSensitive),
        20..=29 =>      Ok(Status::Success),
        30 | 32..=39 => Ok(Status::RedirectTemporary),
        31 =>           Ok(Status::RedirectPermanent),
        41 =>           Ok(Status::FailServerUnavailable),
        40 | 45..=49 => Ok(Status::FailTemporary),
        42 =>           Ok(Status::FailCGIError),
        43 =>           Ok(Status::FailProxyError),
        44 =>           Ok(Status::FailSlowDown),
        50 | 54..=58 => Ok(Status::FailPermanent),
        51 =>           Ok(Status::FailNotFound),
        52 =>           Ok(Status::FailGone),
        53 =>           Ok(Status::FailProxyRequestRefused),
        59 =>           Ok(Status::FailBadRequest),
        60 | 66..=69 => Ok(Status::CertRequiredClient),
        61 =>           Ok(Status::CertRequiredTransient),
        62 =>           Ok(Status::CertRequiredAuthorized),
        63 =>           Ok(Status::CertNotAccepted),
        64 =>           Ok(Status::FutureCertRejected),
        65 =>           Ok(Status::ExpiredCertRejected),
        _ =>            Err(format!("invalid status number: {}", code)),
    }
}
// returns response and content
pub fn get_data(url: &Url) -> Result<(String, String), String> {
    let host = url.host_str().unwrap_or("");
    let urlf = format!("{}:1965", host);
    // get connector
    let connector = TlsConnector::builder()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;
    // get socket address iterator
    let mut addrs_iter = urlf.to_socket_addrs()
        .map_err(|e| e.to_string())?;
    // get socket address from socket address iterator
    let Some(socket_addr) = addrs_iter.next() 
        else {return Err(format!("{}", urlf))};
    // get tcp stream from socket address
    let tcpstream = 
        TcpStream::connect_timeout(&socket_addr, Duration::new(10, 0))
        .map_err(|e| e.to_string())?;
    // get stream from tcp stream
    let mut stream = connector.connect(&host, tcpstream) 
        .map_err(|e| e.to_string())?;
    // write url to stream
    stream.write_all(format!("{}\r\n", url).as_bytes())
        .map_err(|e| e.to_string())?;
    // initialize response vector
    let mut response = vec![];
    // load response vector from stream
    stream.read_to_end(&mut response).map_err(|e| e.to_string())?;
    // find clrf in response vector
    let Some(clrf_idx) = find_clrf(&response) 
        else {return Err("Could not find the clrf".to_string())};
    // separate response from content
    let content = response.split_off(clrf_idx + 2);
    // convert to String
    let content  = String::from_utf8_lossy(&content).to_string();
    let response = String::from_utf8_lossy(&response).to_string();
    Ok((response, content))
}
fn find_clrf(data: &[u8]) -> Option<usize> {
    let clrf = b"\r\n";
    data.windows(clrf.len()).position(|window| window == clrf)
}
#[derive(Deserialize)]
pub struct Config {
    pub init_url:  String,
    pub scroll_at: u16,
    pub colors:    ColorParams,
    pub keys:      KeyParams,
    pub format:    FormatParams,
} impl Config {
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }
    pub fn parse_or_default(text: &str) -> Self {
        toml::from_str(text).unwrap_or(Self::default())
    }
    pub fn default() -> Self {
        Self {
            init_url: "gemini://datapulp.smol.pub/".into(),
            scroll_at: 3,
            colors:    ColorParams::default(),
            keys:      KeyParams::default(),
            format:    FormatParams::default(),
        }
    }
}
#[derive(Deserialize)]
pub struct KeyParams {
    pub global:     char,
    pub load_cfg:   char,
    pub msg_view:   char,
    pub tab_view:   char,
    pub dialog:     DialogKeyParams,
    pub tab:        TabKeyParams,
} impl KeyParams {
    pub fn default() -> Self {
        Self {
            global:     'g',
            load_cfg:   'c',
            msg_view:   'm',
            tab_view:   't',
            dialog:     DialogKeyParams::default(),
            tab:        TabKeyParams::default(),
        }
    }
}
#[derive(Deserialize)]
pub struct TabKeyParams {
    pub move_up:      char,
    pub move_down:    char,
    pub move_left:    char,
    pub move_right:   char,
    pub cycle_left:   char,
    pub cycle_right:  char,
    pub inspect:      char,
    pub delete_tab:   char,
    pub new_tab:      char,
} impl TabKeyParams {
    pub fn default() -> Self {
        Self {
            move_up:      'o',
            move_down:    'i',
            move_left:    'e',
            move_right:   'n',
            cycle_left:   'E',
            cycle_right:  'N',
            inspect:      'w',
            delete_tab:   'v',
            new_tab:      'p',
        }
    }
}
#[derive(Deserialize)]
pub struct DialogKeyParams {
    pub ack: char, 
    pub yes: char, 
    pub no: char
} impl DialogKeyParams {
    pub fn default() -> Self {
        Self {ack: 'y', yes: 'y', no: 'n'}
    }
}
#[derive(Deserialize)]
pub struct FormatParams {
    pub margin:      u16,
    pub list_prefix: String,
    pub heading1:    (u8, u8),
    pub heading2:    (u8, u8),
    pub heading3:    (u8, u8),
} impl FormatParams {
    pub fn default() -> Self {
        Self {
            margin:      2,
            list_prefix: "- ".into(),
            heading1:    (3, 2),
            heading2:    (2, 1),
            heading3:    (1, 0),
        }
    }
}
#[derive(Deserialize)]
pub struct ColorParams {
    pub background: (u8, u8, u8),
    pub banner:     (u8, u8, u8),
    pub dialog:     (u8, u8, u8),
    pub text:       (u8, u8, u8),
    pub heading1:   (u8, u8, u8),
    pub heading2:   (u8, u8, u8),
    pub heading3:   (u8, u8, u8),
    pub link:       (u8, u8, u8),
    pub badlink:    (u8, u8, u8),
    pub quote:      (u8, u8, u8),
    pub list:       (u8, u8, u8),
    pub preformat:  (u8, u8, u8),
} impl ColorParams {
    pub fn default() -> Self {
        Self {
            background: (205, 205, 205),
            dialog:     (  0,   0,   0),
            banner:     (  0,   0,   0),
            text:       (  0,   0,   0),
            heading1:   (  0,   0,   0),
            heading2:   (  0,   0,   0),
            heading3:   (  0,   0,   0),
            link:       (  0,   0,   0),
            badlink:    (  0,   0,   0),
            quote:      (  0,   0,   0),
            list:       (  0,   0,   0),
            preformat:  (  0,   0,   0),
        }
    }
    pub fn get_banner(&self) -> Color {
        let (r, g, b) = self.banner; Color::Rgb {r, g, b}
    }
    pub fn get_dialog(&self) -> Color {
        let (r, g, b) = self.dialog; Color::Rgb {r, g, b}
    }
    pub fn get_background(&self) -> Color {
        let (r, g, b) = self.background; Color::Rgb {r, g, b}
    }
    pub fn from_gem_doc(&self, doc: &GemDoc) -> Vec<DisplayText> {
        doc.doc.iter()
            .map(|(gem_type, text)| self.from_gem_type(gem_type, &text))
            .collect()
    }
    pub fn from_gem_type(&self, gem: &GemType, text: &str) -> DisplayText {
        let ((r, g, b), wrap) = match gem {
            GemType::HeadingOne     => (self.heading1, true),
            GemType::HeadingTwo     => (self.heading2, true),
            GemType::HeadingThree   => (self.heading3, true),
            GemType::Text           => (self.text, true),
            GemType::Quote          => (self.quote, true),
            GemType::ListItem       => (self.list, true),
            GemType::PreFormat      => (self.preformat, false),
            GemType::Link(_, _)     => (self.link, true),
            GemType::BadLink(_)     => (self.badlink, true),
        };
        DisplayText::new(text, Color::Rgb {r, g, b}, wrap)
    }
}
pub fn gem_to_display(gdoc: GemDoc, cfg: &Config, w: usize) -> DisplayDoc {
    let txt = gdoc.doc.iter()
        .map(|(gtype, txt)| {cfg.colors.from_gem_type(gtype, &txt)});
    DisplayDoc::new(txt.collect(), w)
}
pub struct UI {
    pub quit: bool,
    pub scr:  Screen,
    pub ddoc: DisplayDoc,
    pub cfg:  Config,
} impl UI {
    // return default config if error
    fn load_config(path: &str) -> Config {
        fs::read_to_string(path)
            .ok().map(|txt| Config::parse(&txt).ok())
            .flatten().unwrap_or(Config::default())
    }
    pub fn new(path: &str, w: u16, h: u16) -> Self {
        let w = usize::from(w);
        let h = usize::from(h);
        let cfg = Self::load_config(path);
        let scr = Screen::new(w, h);
        let url = url::Url::parse(&cfg.init_url).unwrap();
        let gdoc = GemDoc::new(&url).unwrap();
        let ddoc = gem_to_display(gdoc, &cfg, w);
        Self {quit: false, scr, ddoc, cfg}
    }
    pub fn view(&mut self, writer: &mut impl Write) -> io::Result<()> { 
        writer.queue(Clear(ClearType::All))?;
        self.ddoc.render(&mut self.scr)?;
        self.scr.write(writer)?;
        writer.flush()
    }
    pub fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Key(
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('c'),
                    kind: KeyEventKind::Press, ..
                }
            ) => {
                self.quit = true;
                true
            }
            Event::Resize(w, h) => {
                let w = usize::from(w);
                let h = usize::from(h);
                self.scr.resize(w, h);
                self.ddoc.rewrap(w);
                true
            }
            Event::Key(
                KeyEvent {
                    code: _keycode, 
                    kind: KeyEventKind::Press, ..
                }
            ) => {
                false
            }
            _ => false,
        }
    }
}
fn main() -> io::Result<()> {
    let (w, h) = terminal::size()?;
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.queue(terminal::EnterAlternateScreen)?
        .queue(terminal::DisableLineWrap)?;
    let mut ui = UI::new("gem.toml", w, h);
    ui.view(&mut stdout)?;
    while !ui.quit {
        if ui.update(event::read()?) {
            ui.view(&mut stdout)?;
        }
    }
    terminal::disable_raw_mode()?;
    stdout.queue(terminal::LeaveAlternateScreen)?;
    stdout.flush()
}
