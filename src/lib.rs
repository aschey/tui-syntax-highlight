use std::cell::LazyCell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Widget, Wrap};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet};

static SYNTAXES: OnceLock<Arc<RwLock<SyntaxSet>>> = OnceLock::new();
static THEMES: OnceLock<RwLock<ThemeSet>> = OnceLock::new();
#[cfg(feature = "syntect-assets")]
thread_local! {
    static EXTRA_ASSETS: LazyCell<syntect_assets::assets::HighlightingAssets> =
        LazyCell::new(syntect_assets::assets::HighlightingAssets::from_binary);
}

#[macro_export]
macro_rules! load_syntaxes {
    ($file:expr $(,)?) => {
        $crate::load_syntaxes_from_binary(include_bytes!($file))
    };
}

#[macro_export]
macro_rules! load_themes {
    ($file:expr $(,)?) => {
        $crate::load_themes_from_binary(include_bytes!($file))
    };
}

pub fn load_syntaxes_from_binary(data: &[u8]) {
    SYNTAXES
        .set(Arc::new(RwLock::new(
            syntect::dumps::from_uncompressed_data(data).expect("failed to load syntaxes"),
        )))
        .unwrap();
}

pub fn load_themes_from_binary(data: &[u8]) {
    THEMES.set(syntect::dumps::from_binary(data)).unwrap();
}

#[cfg(feature = "default-syntaxes")]
pub fn load_default_syntaxes() {
    SYNTAXES
        .set(Arc::new(RwLock::new(SyntaxSet::load_defaults_newlines())))
        .unwrap();
}

#[cfg(feature = "default-themes")]
pub fn load_default_themes() {
    THEMES.set(RwLock::new(ThemeSet::load_defaults())).unwrap();
}

#[cfg(feature = "plist-load")]
pub fn add_theme_from_folder(path: impl AsRef<Path>) {
    let mut themes = THEMES.get().unwrap().write().unwrap();
    themes.add_from_folder(path).unwrap();
}

pub fn add_syntax(syntax: SyntaxDefinition) {
    let mut syntaxes = SYNTAXES.get().unwrap().write().unwrap();
    let mut builder = syntaxes.clone().into_builder();
    builder.add(syntax);
    *syntaxes = builder.build();
}

pub fn find_syntax_by_name(name: &str) -> (SyntaxReference, SyntaxSet) {
    #[cfg(feature = "syntect-assets")]
    {
        if let (Some(syntax), syntaxes) = EXTRA_ASSETS.with(|a| {
            let syntaxes = a.get_syntax_set().unwrap();
            (
                syntaxes.find_syntax_by_name(name).cloned(),
                syntaxes.clone(),
            )
        }) {
            return (syntax, syntaxes);
        }
    }
    let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
    (
        syntaxes.find_syntax_by_name(name).unwrap().clone(),
        syntaxes.clone(),
    )
}

pub fn find_syntax_for_file(path: impl AsRef<Path>) -> (SyntaxReference, SyntaxSet) {
    #[cfg(feature = "syntect-assets")]
    {
        if let (Some(syntax), syntaxes) = EXTRA_ASSETS.with(|a| {
            let syntaxes = a.get_syntax_set().unwrap();
            (
                syntaxes.find_syntax_for_file(&path).unwrap().cloned(),
                syntaxes.clone(),
            )
        }) {
            return (syntax, syntaxes);
        }
    }
    let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
    (
        syntaxes
            .find_syntax_for_file(path)
            .unwrap()
            .unwrap()
            .clone(),
        syntaxes.clone(),
    )
}

pub fn find_syntax_by_extension(extension: &str) -> (SyntaxReference, SyntaxSet) {
    #[cfg(feature = "syntect-assets")]
    {
        if let (Some(syntax), syntaxes) = EXTRA_ASSETS.with(|a| {
            let syntaxes = a.get_syntax_set().unwrap();
            (
                syntaxes.find_syntax_by_extension(extension).cloned(),
                syntaxes.clone(),
            )
        }) {
            return (syntax, syntaxes);
        }
    }
    let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
    (
        syntaxes
            .find_syntax_by_extension(extension)
            .unwrap()
            .clone(),
        syntaxes.clone(),
    )
}

pub fn find_syntax_by_first_line(line: &str) -> (SyntaxReference, SyntaxSet) {
    #[cfg(feature = "syntect-assets")]
    {
        if let (Some(syntax), syntaxes) = EXTRA_ASSETS.with(|a| {
            let syntaxes = a.get_syntax_set().unwrap();
            (
                syntaxes.find_syntax_by_first_line(line).cloned(),
                syntaxes.clone(),
            )
        }) {
            return (syntax, syntaxes);
        }
    }
    let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
    (
        syntaxes.find_syntax_by_first_line(line).unwrap().clone(),
        syntaxes.clone(),
    )
}

#[cfg(feature = "yaml-load")]
pub fn add_syntax_from_folder(path: impl AsRef<Path>) {
    let mut syntaxes = SYNTAXES.get().unwrap().write().unwrap();
    let mut builder = syntaxes.clone().into_builder();
    builder.add_from_folder(path, true).unwrap();
    *syntaxes = builder.build();
}

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

fn load_theme(theme: &str) -> Theme {
    #[cfg(feature = "syntect-assets")]
    {
        if let Some(theme) = EXTRA_ASSETS.with(|a| {
            if a.themes().any(|t| t == theme) {
                Some(a.get_theme(theme).clone())
            } else {
                None
            }
        }) {
            return theme.clone();
        }
    }

    THEMES.get().unwrap().read().unwrap().themes[theme].clone()
}

#[derive(Clone, Copy)]
pub enum OverrideBackground {
    Empty,
    Color(ratatui::style::Color),
}

pub struct CodeHighlighter {
    theme: Theme,
    is_ansi_theme: bool,
    override_background: Option<OverrideBackground>,
    include_line_numbers: bool,
    line_number_padding: usize,
    line_number_separator: String,
}

pub struct HighlightedText<'a> {
    pub text: Text<'a>,
    pub background: Option<Color>,
}

impl CodeHighlighter {
    pub fn new(theme: &str) -> Self {
        let theme_obj = load_theme(theme);

        Self {
            theme: theme_obj,
            is_ansi_theme: theme == "ansi",
            override_background: None,
            include_line_numbers: true,
            line_number_padding: 4,
            line_number_separator: "│".to_string(),
            // line_number_style: Style::new().dim(),
        }
    }

    pub fn override_background(mut self, background: OverrideBackground) -> Self {
        self.override_background = Some(background);
        self
    }

    pub fn highlight_file(&self, path: impl AsRef<Path>) -> HighlightedText {
        let (syntax, syntaxes) = find_syntax_for_file(&path);
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let mut highlighter = HighlightLines::new(&syntax, &self.theme);
        let line_number_style = self.get_line_number_style(&mut highlighter, &syntaxes);
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
                    &syntaxes,
                )
                .unwrap();
            formatted.push(highlighted);
            line.clear();
            i += 1;
        }
        HighlightedText {
            text: Text::from_iter(formatted),
            background: line_number_style.bg,
        }
    }

    pub fn highlight_lines(
        &self,
        source: impl IntoLines,
        syntax: Option<(SyntaxReference, SyntaxSet)>,
    ) -> HighlightedText {
        let lines = source.into_lines();
        if lines.is_empty() {
            return HighlightedText {
                text: Text::raw(""),
                background: None,
            };
        }

        let (syntax, syntaxes) = match syntax {
            Some(syntax) => syntax,
            None => find_syntax_by_first_line(&lines[0]),
        };
        let mut highlighter = HighlightLines::new(&syntax, &self.theme);
        let line_number_style = self.get_line_number_style(&mut highlighter, &syntaxes);
        let formatted = lines
            .into_iter()
            .enumerate()
            .map(|(i, line)| {
                self.highlight_line(line, &mut highlighter, i + 1, line_number_style, &syntaxes)
            })
            .collect::<Result<Vec<_>, syntect::Error>>();
        HighlightedText {
            text: Text::from_iter(formatted.unwrap()),
            background: line_number_style.bg,
        }
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
        if self.include_line_numbers {
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

pub struct CodeBlock<'a> {
    inner: Paragraph<'a>,
}

type Vertical = u16;
type Horizontal = u16;

impl<'a> CodeBlock<'a> {
    pub fn new(highlighted: HighlightedText<'a>) -> Self {
        let mut paragraph = Paragraph::new(highlighted.text);
        if let Some(bg) = highlighted.background {
            paragraph = paragraph.bg(bg);
        }

        Self { inner: paragraph }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    pub fn scroll(mut self, offset: (Vertical, Horizontal)) -> Self {
        self.inner = self.inner.scroll(offset);
        self
    }

    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.inner = self.inner.wrap(wrap);
        self
    }
}

impl Widget for CodeBlock<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.inner.render(area, buf)
    }
}
