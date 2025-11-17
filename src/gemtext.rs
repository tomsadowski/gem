// gemtext

// *** BEGIN IMPORTS ***
use url::Url;
use regex::Regex;
use std::str::FromStr;
use crate::{
    util::{ParseError},
    constants,
};
// *** END IMPORTS ***


#[derive(Clone, PartialEq, Debug)]
pub enum Link {
    Gemini(Url, String),
    Gopher(Url, String),
    Http(Url, String),
    Relative(String, String),
    Unknown(Url, String),
}
impl Link {
    pub fn get_text(self) -> String {
        match self {
            Self::Gemini(url, text)      => text,
            Self::Gopher(url, text)      => text,
            Self::Http(url, text)        => text,
            Self::Relative(text1, text2) => text2,
            Self::Unknown(url, text)     => text,
        }
    }
}
impl FromStr for Link {

    type Err = ParseError;

    fn from_str(line: &str) -> Result<Link, ParseError> {

        // get regex
        let Ok(regex) = Regex::new(constants::LINK_REGEX)
            else {return Err(ParseError)};

        // get captures
        let Some(captures) = regex.captures(&line) 
            else {return Err(ParseError)};

        // get url
        let url_str = captures
            .get(1)
            .map_or("", |m| m.as_str())
            .to_string();

        let url_result = Url::parse(&url_str);

        // get label 
        let label_str = captures
            .get(2)
            .map_or("", |m| m.as_str());
        let label = if label_str.is_empty() {
            url_str.clone()
        } 
        else {
            label_str.to_string()
        };

        // return Result
        if let Ok(url) = url_result {
            match url.scheme() {
                constants::GEMINI_SCHEME => 
                    return Ok(Link::Gemini(url, label)),

                constants::GOPHER_SCHEME => 
                    return Ok(Link::Gopher(url, label)),

                constants::HTTP_SCHEME => 
                    return Ok(Link::Http(url, label)),

                constants::HTTPS_SCHEME => 
                    return Ok(Link::Http(url, label)),

                _ => 
                    return Ok(Link::Unknown(url, label)),
            };
        } 
        else if Err(url::ParseError::RelativeUrlWithoutBase) == url_result {
            Ok(Link::Relative(url_str, label))
        } 
        else {
            Err(ParseError) 
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum GemTextLine {
    HeadingOne(String),
    HeadingTwo(String),
    HeadingThree(String),
    Text(String), 
    PreFormat(String),
    Link(Link),
    ListItem(String),
    Quote(String),
} 

impl GemTextLine {

    pub fn parse_doc(lines: Vec<&str>) -> Result<Vec<Self>, String> {

        let mut vec = vec![];
        let mut lines_iter = lines.iter();

        // return empty output if empty input
        let Some(first_line) = lines_iter.next() 
            else {return Ok(vec)};

        let mut preformat_flag = Self::is_toggle(first_line);

        if !preformat_flag {
            // return error if cannot parse formatted line
            let formatted = Self::parse_formatted(first_line)
                .or_else(|e| Err(format!("{}", e)))?;
            vec.push(formatted);
        }

        // parse remaining lines
        for line in lines_iter {
            if Self::is_toggle(line) {
                preformat_flag = !preformat_flag;
            } 
            else if preformat_flag {
                vec.push(Self::PreFormat(line.to_string()));
            }
            else {
                let formatted = Self::parse_formatted(line)
                    .or_else(|e| Err(format!("{}", e)))?;
                vec.push(formatted);
            }
        }

        Ok(vec)
    }

    fn is_toggle(line: &str) -> bool {
        if let Some((symbol, text)) = line.split_at_checked(3) {
            if symbol == constants::TOGGLE_SYMBOL {
                return true
            }
        }
        return false
    }

    fn parse_formatted(line: &str) -> Result<GemTextLine, String> {

        // look for 3 character symbols
        if let Some((symbol, text)) = line.split_at_checked(3) {
            if symbol == constants::HEADING_3_SYMBOL {
                return Ok(GemTextLine::HeadingThree(text.to_string()))
            }
        }

        // look for 2 character symbols
        if let Some((symbol, text)) = line.split_at_checked(2) {
            if symbol == constants::LINK_SYMBOL {
                let link = Link::from_str(text)
                    .or_else(|e| Err(format!("could not parse link {:?}", e)))?;
                return Ok(GemTextLine::Link(link))
            }
            if symbol == constants::HEADING_2_SYMBOL {
                return Ok(GemTextLine::HeadingTwo(text.to_string()))
            }
        }

        // look for 1 character symbols
        if let Some((symbol, text)) = line.split_at_checked(1) {
            if symbol == constants::QUOTE_SYMBOL {
                return Ok(GemTextLine::Quote(text.to_string()))
            }
            if symbol == constants::LIST_ITEM_SYMBOL {
                return Ok(GemTextLine::ListItem(text.to_string()))
            }
            if symbol == constants::HEADING_1_SYMBOL {
                return Ok(GemTextLine::HeadingOne(text.to_string()))
            }
        }

        Ok(GemTextLine::Text(line.to_string()))
    }
}
