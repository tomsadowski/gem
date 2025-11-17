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
    pub heading3: Style,
    pub heading2: Style,
    pub heading1: Style,
    pub link: Style,
    pub quote: Style,
    pub preformat: Style,
    pub text: Style,
}

// Implements Widget by parsing ModelText onto a Vec of Spans
#[derive(Clone, Debug)]
pub struct DisplayModelText<'a> {
    pub source:  ModelText,
    pub text:    Vec<Span<'a>>,
}
impl<'a> DisplayModelText<'a> {
    fn parse_gemtext(text: &Vec<GemTextLine>, styles: LineStyles) -> Vec<Span<'a>> {
        vec![]
    }
    fn parse_text(text: &str, styles: LineStyles) -> Vec<Span<'a>> {
        vec![]
    }
    pub fn new(text: ModelText, styles: LineStyles) -> Self {
        let spans = match text {
            ModelText::GemText(ref lines) => 
                Self::parse_gemtext(lines, styles),
            ModelText::String(ref line)   => 
                Self::parse_text(line, styles),
        };
        Self {
            source:  text,
            text:    spans,
        }
    }
}

// Implements Widget by projecting Model onto Widgets
#[derive(Clone, Debug)]
pub struct DisplayModel<'a> {
    pub source:  Model,
    pub text:    DisplayModelText<'a>,
}
impl<'a> DisplayModel<'a> {
    pub fn new(model: Model, styles: LineStyles) -> Self {
        Self {
            source: model.clone(),
            text: DisplayModelText::new(model.text, styles),
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
