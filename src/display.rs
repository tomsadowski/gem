// display

// *** BEGIN IMPORTS ***
use crate::{
    model::{
        Model, 
        ModelText
    },
    gemtext::{
        GemTextLine,
    }
};
use ratatui::{
    prelude::*, 
    text::{
        Line,
        Span,
        Text
    },
    style::{
        Color, 
        Style, 
        Modifier,
    },
    widgets::{
        Paragraph,
        Wrap
    },
};
// *** END IMPORTS ***


#[derive(Clone, Debug)]
pub struct LineStyles {
    pub heading_one: Style,
    pub heading_two: Style,
    pub heading_three: Style,
    pub link: Style,
    pub list_item: Style,
    pub quote: Style,
    pub preformat: Style,
    pub text: Style,
    pub plaintext: Style,
}

// Implements Widget by parsing ModelText onto a Vec of Spans
#[derive(Clone, Debug)]
pub struct GemTextSpan<'a> {
    pub source: GemTextLine,
    pub span:   Span<'a>,
}
impl<'a> GemTextSpan<'a> {
    fn new(text: &GemTextLine, styles: &LineStyles) -> Self {
        let span = match text.clone() {
            GemTextLine::Text(s) => {
                Span::from(s).style(styles.text)
            }
            GemTextLine::HeadingOne(s) => {
                Span::from(s).style(styles.heading_one)
            }
            GemTextLine::HeadingTwo(s) => {
                Span::from(s).style(styles.heading_two)
            }
            GemTextLine::HeadingThree(s) => {
                Span::from(s).style(styles.heading_three)
            }
            GemTextLine::Link(link) => {
                Span::from(link.get_text()).style(styles.link)
            }
            GemTextLine::Quote(s) => {
                Span::from(s).style(styles.quote)
            }
            GemTextLine::ListItem(s) => {
                Span::from(s).style(styles.list_item)
            }
            GemTextLine::PreFormat(s) => {
                Span::from(s).style(styles.preformat)
            }
        };
        Self {
            source: text.clone(),
            span:   span,
        }
    }
}

// Implements Widget by parsing ModelText onto a Vec of Spans
#[derive(Clone, Debug)]
pub struct PlainTextSpan<'a> {
    pub source: String,
    pub span:   Span<'a>,
}
impl<'a> PlainTextSpan<'a> {
    fn new(text: &'a str, styles: &LineStyles) -> Self {
        Self {
            source: String::from(text),
            span:   Span::from(text).style(styles.plaintext),
        }
    }
}

// Implements Widget by parsing ModelText onto a Vec of Spans
#[derive(Clone, Debug)]
pub enum DisplayModelText<'a> {
    GemText(Vec<GemTextSpan<'a>>),
    PlainText(Vec<PlainTextSpan<'a>>),
}
impl<'a> DisplayModelText<'a> {
    pub fn new(text: &'a ModelText, styles: LineStyles) -> Self {
        match text {
            ModelText::GemText(lines) => 
                Self::GemText(
                    lines
                        .iter()
                        .map(|line| GemTextSpan::new(line, &styles))
                        .collect()
                ),
            ModelText::PlainText(lines)  => 
                Self::PlainText(
                    lines
                        .iter()
                        .map(|line| PlainTextSpan::new(line, &styles))
                        .collect()
                ),
        }
    }
}

// Implements Widget by projecting Model onto Widgets
#[derive(Clone, Debug)]
pub struct DisplayModel<'a> {
    pub source: Model,
    pub text:   DisplayModelText<'a>,
}
impl<'a> DisplayModel<'a> {
    pub fn new(model: &'a Model, styles: LineStyles) -> Self {
        Self {
            source: model.clone(),
            text:   DisplayModelText::new(&model.text, styles),
        }
    }
}
impl<'a> Widget for &DisplayModel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = format!("{:#?}", self.text);
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll((self.source.y, self.source.x))
            .render(area, buf);
    }
}
