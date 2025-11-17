// styles


// *** BEGIN IMPORTS ***
use ratatui::{
    style::{
        Color, 
        Style, 
        Modifier,
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
impl LineStyles {
    pub fn new() -> Self {
        let heading_one_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));
        let heading_two_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));
        let heading_three_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));
        let link_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));
        let text_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));
        let list_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));
        let quote_style = Style::new()
                .fg(Color::Rgb(128, 64, 32))
                .bg(Color::Rgb(16,  32, 32));

        Self {
            heading_one:   heading_one_style,
            heading_two:   heading_two_style,
            heading_three: heading_three_style,
            link:          link_style,
            list_item:     list_style,
            quote:         quote_style,
            preformat:     text_style,
            plaintext:     text_style,
            text:          text_style,
        }
    }
}

