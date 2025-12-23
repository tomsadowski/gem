// gem/src/util
use std::{
    time::{Duration}, 
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
};
use url::Url;
use native_tls::TlsConnector;

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
pub fn split_whitespace_once(line: &str) -> (&str, &str) {
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
// returns response and content
pub fn get_data(url: &Url) -> Result<(String, String), String> {
    let host = url.host_str().unwrap_or("");
    let urlf = format!("{}:1965", host);
    let failmsg = "Could not connect to ";

    // get connector
    let connector = TlsConnector::builder()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .or_else(|e| Err(format!("{}{}\n{}", failmsg, urlf, e)))?;

    // get socket address iterator
    let mut addrs_iter = urlf.to_socket_addrs()
        .or_else(|e| Err(format!("{}{}\n{}", failmsg, urlf, e)))?;

    // get socket address from socket address iterator
    let Some(socket_addr) = addrs_iter.next() 
        else {return Err(format!("Could not connect to {}", urlf))};

    // get tcp stream from socket address
    let tcpstream = TcpStream::connect_timeout
        (&socket_addr, Duration::new(10, 0))
        .or_else(|e| Err(format!("Could not connect to {}\n{}", urlf, e)))?;

    // get stream from tcp stream
    let mut stream = connector.connect(&host, tcpstream) 
        .or_else(|e| Err(format!("Could not connect to {}\n{}", urlf, e)))?;

    // write url to stream
    stream.write_all(format!("{}\r\n", url).as_bytes())
        .or_else(|e| Err(format!("Could not write to {}\n{}", url, e)))?;

    // initialize response vector
    let mut response = vec![];

    // load response vector from stream
    stream.read_to_end(&mut response)
        .or_else(|e| Err(format!("Could not read {}\n{}", url, e)))?;

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
