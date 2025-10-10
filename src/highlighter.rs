use std::borrow::Cow;
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
#[cfg(feature = "profile")]
use termprofile::TermProfile;

use crate::Converter;

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
    #[cfg(feature = "profile")]
    profile: TermProfile,
    highlight_ranges: Vec<Range<usize>>,
    highlight_style: Style,
    converter: Converter,
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
            .field("converter", &self.converter)
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
            #[cfg(feature = "profile")]
            profile: TermProfile::TrueColor,
            highlight_ranges: Vec::new(),
            highlight_style: Style::new(),
            converter: Converter::new(),
        }
    }

    #[cfg(feature = "profile")]
    pub fn with_profile(theme: Theme, profile: TermProfile) -> Self {
        let mut this = Self::new(theme);
        this.profile = profile;
        this.converter = Converter::with_profile(profile);
        this
    }

    pub fn override_background<C>(mut self, background: C) -> Self
    where
        C: Into<Color>,
    {
        let background = background.into();
        self.override_background = Some(self.adapt_color(background).unwrap_or(Color::Reset));
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
        self.line_number_style = Some(self.adapt_style(style));
        self
    }

    pub fn line_number_separator_style(mut self, style: Style) -> Self {
        self.line_number_separator_style = Some(self.adapt_style(style));
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
        self.highlight_style = self.adapt_style(style);
        self
    }

    pub fn gutter_template<F>(mut self, template: F) -> Self
    where
        F: Fn(usize, Style) -> Vec<Span<'static>> + Send + Sync + 'static,
    {
        self.gutter_template = Some(Arc::new(template));
        self
    }

    pub fn create_line_highlighter(&self, syntax: &SyntaxReference) -> HighlightLines<'_> {
        HighlightLines::new(syntax, &self.theme)
    }

    pub fn highlight_reader<R>(
        &self,
        reader: R,
        syntax: &SyntaxReference,
        syntaxes: &SyntaxSet,
    ) -> Result<Text<'static>, crate::Error>
    where
        R: io::Read,
    {
        let mut reader = BufReader::new(reader);
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let line_number_style = self.calculate_line_number_style();
        let mut line = String::new();
        let mut formatted = Vec::new();
        let mut i = 1;
        while reader.read_line(&mut line).map_err(crate::Error::Source)? > 0 {
            let highlighted =
                self.highlight_line(&line, &mut highlighter, i, line_number_style, syntaxes)?;
            formatted.push(highlighted);
            line.clear();
            i += 1;
        }
        Ok(Text::from_iter(formatted))
    }

    pub fn highlight_lines<'a, T>(
        &self,
        source: T,
        syntax: &SyntaxReference,
        syntaxes: &SyntaxSet,
    ) -> Result<Text<'static>, crate::Error>
    where
        T: IntoIterator<Item = &'a str>,
    {
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let line_number_style = self.calculate_line_number_style();
        let formatted: Result<Vec<_>, crate::Error> = source
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                self.highlight_line(line, &mut highlighter, i + 1, line_number_style, syntaxes)
            })
            .collect();
        let formatted = formatted?;
        Ok(Text::from_iter(formatted))
    }

    pub fn highlight_line(
        &self,
        line: &str,
        highlighter: &mut HighlightLines,
        line_number: usize,
        line_number_style: Style,
        syntaxes: &SyntaxSet,
    ) -> Result<Line<'static>, crate::Error> {
        let line: Cow<_> = if line.ends_with("\n") {
            line.into()
        } else {
            (line.to_string() + "\n").into()
        };
        let regions = highlighter
            .highlight_line(&line, syntaxes)
            .map_err(crate::Error::Highlight)?;
        Ok(self.to_line(&regions, line_number, line_number_style))
    }

    pub fn calculate_line_number_style(&self) -> Style {
        if let Some(style) = self.line_number_style {
            return style;
        }
        let mut style = Style::new();
        if let Some(fg) = self
            .theme
            .settings
            .gutter_foreground
            .and_then(|fg| self.converter.syntect_color_to_tui(fg))
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
            .and_then(|bg| self.converter.syntect_color_to_tui(bg))
        {
            style = style.bg(bg);
        }
        self.adapt_style(style)
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

    fn to_line(
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
        self.apply_background(line)
    }

    fn adapt_style(&self, style: Style) -> Style {
        #[cfg(feature = "profile")]
        return self.profile.adapt_style(style);
        #[cfg(not(feature = "profile"))]
        return style;
    }

    fn adapt_color(&self, color: Color) -> Option<Color> {
        #[cfg(feature = "profile")]
        return self.profile.adapt_color(color);

        #[cfg(not(feature = "profile"))]
        return Some(color);
    }

    fn apply_background<'a, S>(&self, item: S) -> S
    where
        S: Stylize<'a, S>,
    {
        if let Some(bg) = self.override_background {
            return item.bg(bg);
        };
        if let Some(bg) = self
            .theme
            .settings
            .background
            .and_then(|bg| self.converter.syntect_color_to_tui(bg))
        {
            return item.bg(bg);
        }
        item
    }

    fn syntect_style_to_tui(&self, style: syntect::highlighting::Style) -> ratatui::style::Style {
        let mut tui_style = self.converter.syntect_style_to_tui(style);

        if let Some(bg) = self.override_background {
            tui_style = tui_style.bg(bg);
        }
        tui_style
    }
}
