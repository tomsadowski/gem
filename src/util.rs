// gem/src/util

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
        start = line.floor_char_boundary(start);
        wrapped.push(String::from(&line[start..]));
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
