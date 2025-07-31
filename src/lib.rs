use std::io::{self, BufRead, BufReader};
use std::ops::{Deref, DerefMut};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};
pub use syntect;
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub trait IntoLines {
    fn into_lines(self) -> Vec<String>;
}

impl IntoLines for Vec<String> {
    fn into_lines(self) -> Vec<String> {
        self
    }
}

impl IntoLines for Vec<&str> {
    fn into_lines(self) -> Vec<String> {
        self.into_iter().map(|s| s.into()).collect()
    }
}

impl IntoLines for String {
    fn into_lines(self) -> Vec<String> {
        self.split('\n').map(|s| s.into()).collect()
    }
}

impl IntoLines for &str {
    fn into_lines(self) -> Vec<String> {
        self.split('\n').map(|s| s.into()).collect()
    }
}

#[derive(Clone, Copy)]
pub enum OverrideBackground {
    Empty,
    Color(ratatui::style::Color),
}

pub struct HighlightedText<'a>(pub Text<'a>);

impl<'a> HighlightedText<'a> {
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

impl<'a> Deref for HighlightedText<'a> {
    type Target = Text<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HighlightedText<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Widget for HighlightedText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.0.render(area, buf);
    }
}

pub struct CodeHighlighter {
    theme: Theme,
    override_background: Option<OverrideBackground>,
    line_numbers: bool,
    line_number_padding: usize,
    line_number_separator: String,
    syntaxes: SyntaxSet,
    is_ansi_theme: bool,
}

impl CodeHighlighter {
    pub fn new(theme: Theme, syntaxes: SyntaxSet) -> Self {
        let is_ansi_theme = theme
            .name
            .as_deref()
            .unwrap_or_default()
            .eq_ignore_ascii_case("ansi");
        Self {
            theme,
            override_background: None,
            line_numbers: true,
            line_number_padding: 4,
            line_number_separator: "│".to_string(),
            syntaxes,
            is_ansi_theme,
        }
    }

    pub fn syntaxes(&self) -> &SyntaxSet {
        &self.syntaxes
    }

    pub fn override_background(mut self, background: OverrideBackground) -> Self {
        self.override_background = Some(background);
        self
    }

    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    pub fn highlight_reader<R>(&self, reader: R, syntax: &SyntaxReference) -> HighlightedText
    where
        R: io::Read,
    {
        let mut reader = BufReader::new(reader);
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let line_number_style = self.get_line_number_style(&mut highlighter, &self.syntaxes);
        let mut line = String::new();
        let mut formatted = Vec::new();
        let mut i = 1;
        while reader.read_line(&mut line).unwrap() > 0 {
            let highlighted = self
                .highlight_line(
                    line.clone(),
                    &mut highlighter,
                    i,
                    line_number_style,
                    &self.syntaxes,
                )
                .unwrap();
            formatted.push(highlighted);
            line.clear();
            i += 1;
        }
        self.to_text(Text::from_iter(formatted), line_number_style.bg)
    }

    pub fn highlight_lines(
        &self,
        source: impl IntoLines,
        syntax: &SyntaxReference,
    ) -> HighlightedText {
        let lines = source.into_lines();
        if lines.is_empty() {
            return HighlightedText(Text::raw(""));
        }

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let line_number_style = self.get_line_number_style(&mut highlighter, &self.syntaxes);
        let formatted = lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                self.highlight_line(
                    line,
                    &mut highlighter,
                    i + 1,
                    line_number_style,
                    &self.syntaxes,
                )
            })
            .collect::<Result<Vec<_>, syntect::Error>>();
        self.to_text(Text::from_iter(formatted.unwrap()), line_number_style.bg)
    }

    fn to_text<'a>(&self, text: Text<'a>, bg: Option<Color>) -> HighlightedText<'a> {
        if let Some(OverrideBackground::Color(color)) = self.override_background {
            return HighlightedText(text.bg(color));
        };
        if let Some(bg) = bg {
            return HighlightedText(text.bg(bg));
        }
        HighlightedText(text)
    }

    fn highlight_line(
        &self,
        line: String,
        highlighter: &mut HighlightLines,
        line_number: usize,
        line_number_style: Style,
        syntaxes: &SyntaxSet,
    ) -> Result<Line<'static>, syntect::Error> {
        let line = if line.ends_with('\n') {
            line
        } else {
            format!("{line}\n")
        };

        let regions = highlighter.highlight_line(&line, syntaxes)?;

        Ok(if self.is_ansi_theme {
            self.to_tui_text_ansi_theme(&regions[..], line_number, line_number_style)
        } else {
            self.to_tui_text(&regions, line_number, line_number_style)
        })
    }

    fn get_line_number_style(
        &self,
        highlighter: &mut HighlightLines,
        syntaxes: &SyntaxSet,
    ) -> Style {
        if self.is_ansi_theme {
            Style::new()
        } else {
            let style = highlighter
                .highlight_line(" \n", syntaxes)
                .unwrap()
                .first()
                .unwrap()
                .0;
            self.syntect_style_to_tui(&style)
        }
    }

    fn get_initial_spans(
        &self,
        line_number: usize,
        line_number_style: Style,
    ) -> Vec<Span<'static>> {
        if self.line_numbers {
            vec![
                Span::from(format!(
                    "{line_number:^width$}{} ",
                    self.line_number_separator,
                    width = self.line_number_padding
                ))
                .style(line_number_style)
                .dim(),
            ]
        } else {
            vec![]
        }
    }

    fn to_tui_text_ansi_theme(
        &self,
        v: &[(syntect::highlighting::Style, &str)],
        line_number: usize,
        line_number_style: Style,
    ) -> Line<'static> {
        let mut spans = self.get_initial_spans(line_number, line_number_style);
        for &(ref style, mut text) in v.iter() {
            let ends_with_newline = text.ends_with('\n');
            if ends_with_newline {
                text = &text[..text.len() - 1];
            }

            let fg = if style.foreground.a == 0 {
                ansi_color_to_tui(style.foreground.r)
            } else if style.foreground.a == 1 {
                ratatui::style::Color::default()
            } else {
                ratatui::style::Color::Rgb(
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                )
            };
            let mut tui_style = ratatui::style::Style::new().fg(fg);
            if let Some(OverrideBackground::Color(bg)) = self.override_background {
                tui_style = tui_style.bg(bg);
            }
            tui_style = tui_style.add_modifier(syntect_modifiers_to_tui(&style.font_style));
            spans.push(Span::styled(text.to_string(), tui_style));
        }
        Line::from_iter(spans)
    }

    fn to_tui_text(
        &self,
        v: &[(syntect::highlighting::Style, &str)],
        line_number: usize,
        line_number_style: Style,
    ) -> Line<'static> {
        let mut spans = self.get_initial_spans(line_number, line_number_style);
        for &(ref style, mut text) in v.iter() {
            let ends_with_newline = text.ends_with('\n');
            if ends_with_newline {
                text = &text[..text.len() - 1];
            }

            let tui_style = self.syntect_style_to_tui(style);

            spans.push(Span::styled(text.to_string(), tui_style));
        }

        Line::from_iter(spans)
    }

    fn syntect_style_to_tui(&self, style: &syntect::highlighting::Style) -> ratatui::style::Style {
        let mut tui_style = ratatui::style::Style::new()
            .fg(ratatui::style::Color::Rgb(
                style.foreground.r,
                style.foreground.b,
                style.foreground.g,
            ))
            .add_modifier(syntect_modifiers_to_tui(&style.font_style));
        if let Some(bg) = self.override_background {
            match bg {
                OverrideBackground::Empty => {}
                OverrideBackground::Color(color) => {
                    tui_style = tui_style.bg(color);
                }
            };
        } else {
            tui_style = tui_style.bg(ratatui::style::Color::Rgb(
                style.background.r,
                style.background.g,
                style.background.b,
            ))
        }
        tui_style
    }
}

fn syntect_modifiers_to_tui(style: &syntect::highlighting::FontStyle) -> ratatui::style::Modifier {
    let mut modifier = ratatui::style::Modifier::empty();
    if style.intersects(syntect::highlighting::FontStyle::BOLD) {
        modifier |= ratatui::style::Modifier::BOLD;
    }
    if style.intersects(syntect::highlighting::FontStyle::ITALIC) {
        modifier |= ratatui::style::Modifier::ITALIC;
    }
    if style.intersects(syntect::highlighting::FontStyle::UNDERLINE) {
        modifier |= ratatui::style::Modifier::UNDERLINED;
    }
    modifier
}

fn ansi_color_to_tui(value: u8) -> ratatui::style::Color {
    match value {
        0x00 => ratatui::style::Color::Black,
        0x01 => ratatui::style::Color::Red,
        0x02 => ratatui::style::Color::Green,
        0x03 => ratatui::style::Color::Yellow,
        0x04 => ratatui::style::Color::Blue,
        0x05 => ratatui::style::Color::Magenta,
        0x06 => ratatui::style::Color::Cyan,
        0x07 => ratatui::style::Color::Gray,
        0x08 => ratatui::style::Color::DarkGray,
        0x09 => ratatui::style::Color::LightRed,
        0x0A => ratatui::style::Color::LightGreen,
        0x0B => ratatui::style::Color::LightYellow,
        0x0C => ratatui::style::Color::LightBlue,
        0x0D => ratatui::style::Color::LightMagenta,
        0x0E => ratatui::style::Color::LightCyan,
        0x0F => ratatui::style::Color::White,
        _ => ratatui::style::Color::Gray,
    }
}
