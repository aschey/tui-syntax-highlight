use std::fmt::Debug;
use std::io::{self, BufRead, BufReader};
use std::ops::Range;
use std::sync::Arc;

use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
pub use syntect;
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::{ColorMode, HighlightedText, IntoLines, syntect_color_to_tui};

type GutterFn = dyn Fn(usize, Style) -> Vec<Span<'static>> + Send + Sync;

#[derive(Clone)]
pub struct Highlighter {
    theme: Theme,
    override_background: Option<Color>,
    line_number_style: Option<Style>,
    line_number_separator_style: Option<Style>,
    gutter_template: Option<Arc<GutterFn>>,
    line_numbers: bool,
    line_number_padding: usize,
    line_number_separator: String,
    color_mode: ColorMode,
    highlight_ranges: Vec<Range<usize>>,
    highlight_style: Style,
}

impl Debug for Highlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Highlighter")
            .field("theme", &self.theme)
            .field("override_background", &self.override_background)
            .field("line_number_style", &self.line_number_style)
            .field(
                "line_number_separator_style",
                &self.line_number_separator_style,
            )
            .field("gutter_template", &"<fn>")
            .field("line_numbers", &self.line_numbers)
            .field("line_number_padding", &self.line_number_padding)
            .field("line_number_separator", &self.line_number_separator)
            .field("color_mode", &self.color_mode)
            .finish()
    }
}

impl Highlighter {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            override_background: None,
            line_number_style: None,
            line_number_separator_style: None,
            gutter_template: None,
            line_numbers: true,
            line_number_padding: 4,
            line_number_separator: "│".to_string(),
            color_mode: ColorMode::TrueColor,
            highlight_ranges: Vec::new(),
            highlight_style: Style::new(),
        }
    }

    pub fn override_background<C>(mut self, background: C) -> Self
    where
        C: Into<Color>,
    {
        self.override_background = Some(background.into());
        self
    }

    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    pub fn line_number_padding(mut self, padding: usize) -> Self {
        self.line_number_padding = padding;
        self
    }

    pub fn line_number_style(mut self, style: Style) -> Self {
        self.line_number_style = Some(style);
        self
    }

    pub fn line_number_separator_style(mut self, style: Style) -> Self {
        self.line_number_separator_style = Some(style);
        self
    }

    pub fn line_number_separator<T>(mut self, separator: T) -> Self
    where
        T: Into<String>,
    {
        self.line_number_separator = separator.into();
        self
    }

    pub fn highlight_range(mut self, range: Range<usize>) -> Self {
        self.highlight_ranges.push(range);
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    pub fn gutter_template<F>(mut self, template: F) -> Self
    where
        F: Fn(usize, Style) -> Vec<Span<'static>> + Send + Sync + 'static,
    {
        self.gutter_template = Some(Arc::new(template));
        self
    }

    pub fn highlight_reader<R>(
        &self,
        reader: R,
        syntax: &SyntaxReference,
        syntaxes: &SyntaxSet,
    ) -> Result<HighlightedText<'static>, crate::Error>
    where
        R: io::Read,
    {
        let mut reader = BufReader::new(reader);
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let line_number_style = self.get_line_number_style();
        let mut line = String::new();
        let mut formatted = Vec::new();
        let mut i = 1;
        while reader.read_line(&mut line).map_err(crate::Error::Source)? > 0 {
            let highlighted = self.highlight_line(
                line.clone(),
                &mut highlighter,
                i,
                line_number_style,
                syntaxes,
            )?;
            formatted.push(highlighted);
            line.clear();
            i += 1;
        }
        Ok(self.to_text(Text::from_iter(formatted)))
    }

    pub fn highlight_lines<T>(
        &self,
        source: T,
        syntax: &SyntaxReference,
        syntaxes: &SyntaxSet,
    ) -> Result<HighlightedText<'static>, crate::Error>
    where
        T: IntoLines,
    {
        let lines = source.into_lines();
        if lines.is_empty() {
            return Ok(HighlightedText(Text::raw("")));
        }

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let line_number_style = self.get_line_number_style();
        let formatted: Result<Vec<_>, crate::Error> = lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                self.highlight_line(line, &mut highlighter, i + 1, line_number_style, syntaxes)
            })
            .collect();
        let formatted = formatted?;
        Ok(self.to_text(Text::from_iter(formatted)))
    }

    fn to_text(&self, text: Text<'static>) -> HighlightedText<'static> {
        if let Some(bg) = self.override_background {
            return HighlightedText(text.bg(bg));
        };
        if let Some(bg) = self
            .theme
            .settings
            .background
            .and_then(|bg| syntect_color_to_tui(bg, self.color_mode))
        {
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
    ) -> Result<Line<'static>, crate::Error> {
        let line = if line.ends_with('\n') {
            line
        } else {
            format!("{line}\n")
        };

        let regions = highlighter
            .highlight_line(&line, syntaxes)
            .map_err(crate::Error::Highlight)?;
        Ok(self.to_tui_text(&regions, line_number, line_number_style))
    }

    fn get_line_number_style(&self) -> Style {
        if let Some(style) = self.line_number_style {
            return style;
        }
        let mut style = Style::new();
        if let Some(fg) = self
            .theme
            .settings
            .gutter_foreground
            .and_then(|fg| syntect_color_to_tui(fg, self.color_mode))
        {
            style = style.fg(fg);
        } else {
            style = style.dark_gray();
        }
        if let Some(bg) = self.override_background {
            style = style.bg(bg);
        } else if let Some(bg) = self
            .theme
            .settings
            .background
            .and_then(|bg| syntect_color_to_tui(bg, self.color_mode))
        {
            style = style.bg(bg);
        }
        style
    }

    fn get_initial_spans(
        &self,
        line_number: usize,
        line_number_style: Style,
    ) -> Vec<Span<'static>> {
        if let Some(template) = &self.gutter_template {
            return template(line_number, line_number_style);
        }
        if self.line_numbers {
            let line_number = line_number.to_string();
            let spaces = self
                .line_number_padding
                .saturating_sub(line_number.len())
                // 2 extra spaces for left/right padding
                .saturating_sub(2);
            vec![
                Span::styled(" ".repeat(spaces), line_number_style),
                Span::styled(line_number, line_number_style),
                Span::styled(" ", line_number_style),
                Span::styled(
                    self.line_number_separator.clone(),
                    self.line_number_separator_style
                        .unwrap_or(line_number_style),
                ),
                Span::styled(" ", line_number_style),
            ]
        } else {
            vec![]
        }
    }

    fn to_tui_text(
        &self,
        v: &[(syntect::highlighting::Style, &str)],
        line_number: usize,
        line_number_style: Style,
    ) -> Line<'static> {
        let mut spans = self.get_initial_spans(line_number, line_number_style);

        let highlight_row = self
            .highlight_ranges
            .iter()
            .any(|r| r.contains(&line_number));

        for &(ref style, mut text) in v.iter() {
            let ends_with_newline = text.ends_with('\n');
            if ends_with_newline {
                text = &text[..text.len() - 1];
            }

            let mut tui_style = self.syntect_style_to_tui(*style);
            if highlight_row {
                tui_style = tui_style.patch(self.highlight_style);
            }

            spans.push(Span::styled(text.to_string(), tui_style));
        }

        let mut line = Line::from_iter(spans);
        if highlight_row {
            line = line.patch_style(self.highlight_style);
        }
        line
    }

    fn syntect_style_to_tui(&self, style: syntect::highlighting::Style) -> ratatui::style::Style {
        let mut tui_style = crate::syntect_style_to_tui(style, self.color_mode);

        if let Some(bg) = self.override_background {
            tui_style = tui_style.bg(bg);
        }
        tui_style
    }
}
