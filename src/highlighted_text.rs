
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget};
pub use syntect;

pub struct HighlightedText<'a>(pub(crate) Text<'a>);

impl<'a> HighlightedText<'a> {
    pub fn into_text(self) -> Text<'a> {
        self.0
    }
    pub fn into_paragraph(self) -> Paragraph<'a> {
        let bg = self.0.style.bg;
        let paragraph = Paragraph::new(self.0);
        if let Some(bg) = bg {
            paragraph.bg(bg)
        } else {
            paragraph
        }
    }
}

impl Widget for HighlightedText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(bg) = self.0.style.bg {
            buf.set_style(area, Style::new().bg(bg));
        }
        self.0.render(area, buf);
    }
}
