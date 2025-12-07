// text

use crate::{
    gemini::Status,
    gemini::GemTextLine,
    gemini::GemTextData,
    view::LineStyles,
};
use crossterm::{
    style::Colors,
};


#[derive(Clone, Debug)]
pub struct GemTextBlock {
    data:    GemTextData,
    text:    Vec<String>,
    style:   Colors,
    height:  usize,
    current: usize,
}
impl GemTextBlock {
    fn new(line: &GemTextLine, styles: &LineStyles, size: (u16, u16)) -> Self {
        let style = styles.get_colors(line.data.clone());
        
        let width = usize::from(size.0);
        let lines: Vec<String> = 
            line.text.splitn(width, ' ')
                .map(|s| s.to_string())
                .collect(); 

        Self {
            data:    line.data.clone(),
            style:   style.clone(),
            height:  lines.len(),
            text:    lines,
            current: 0,
        }
    }

    pub fn get_text_under_cursor(&self) -> (GemTextData, String) {
        (self.data.clone(), self.text[self.current].clone())
    }

    pub fn move_cursor_up(&mut self) -> bool {
        if self.current > 0 { 
            self.current -= 1;
            return true;
        }
        else {
            return false;
        }
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.current < self.text.len() - 1 {
            self.current += 1;
            return true;
        }
        else {
            return false;
        }
    }
}

// the model's main viewport
#[derive(Clone, Debug)]
pub struct GemTextView {
    text:    Vec<GemTextBlock>,
    styles:  LineStyles,
    size:    (u16, u16),
    current: usize,
}
impl GemTextView {
    pub fn new(content: String, styles: &LineStyles, size: (u16, u16)) -> Self {
        let text = 
                GemTextLine::parse_doc(
                    content
                        .lines()
                        .collect()
                )
                .unwrap()
                .iter()
                .map(|line| GemTextBlock::new(line, &styles, size))
                .collect();
        Self {
            text:    text,
            styles:  styles.clone(),
            size:    size,
            current: 0,
        }
    }

    pub fn update_from_response(self, status: Status, content: String) -> Self {
        let text = match status {
            Status::Success(variant, meta) => {
                if meta.starts_with("text/") {
                    content
                } 
                else {
                    format!("nontext media encountered {:?}: {:?}", variant, meta)
                }
            }
            Status::InputExpected(variant, meta) => {
                format!("Input Expected {:?}: {:?}", variant, meta)
            }
            Status::TemporaryFailure(variant, meta) => {
                format!("Temporary Failure {:?}: {:?}", variant, meta)
            }
            Status::PermanentFailure(variant, meta) => {
                format!("Permanent Failure {:?}: {:?}", variant, meta)
            }
            Status::Redirect(_variant, new_url) => {
                format!("Redirect to: {}?", new_url)
            }
            Status::ClientCertRequired(_variant, meta) => {
                format!("Certificate required: {}", meta)
            }
        };

        Self::new(text, &self.styles, self.size)
    }

    pub fn get_text_under_cursor(&self) -> (GemTextData, String) {
        self.text[self.current].get_text_under_cursor()
    }

    pub fn move_cursor_up(&mut self) -> bool {
        if self.text[self.current].move_cursor_up() {
            return true;
        }
        else if self.current == 0 {
            return false;
        }
        else {
            self.current -= 1;
            return self.move_cursor_up();
        }
    }

    pub fn move_cursor_down(&mut self) -> bool {
        if self.text[self.current].move_cursor_down() {
            return true;
        }
        else if self.current == self.text.len() - 1 {
            return false;
        }
        else {
            self.current += 1;
            return self.move_cursor_down();
        }
    }
}
