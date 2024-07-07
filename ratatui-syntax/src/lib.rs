use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

use ratatui::text::{Line, Span, Text};
use syntect::easy::{HighlightFile, HighlightLines};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet};

pub(crate) static SYNTAXES: OnceLock<Arc<RwLock<SyntaxSet>>> = OnceLock::new();
static THEMES: OnceLock<RwLock<ThemeSet>> = OnceLock::new();
static ANSI_THEME: OnceLock<Theme> = OnceLock::new();

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

pub fn load_default_syntaxes() {
    SYNTAXES
        .set(Arc::new(RwLock::new(SyntaxSet::load_defaults_newlines())))
        .unwrap();
}

#[cfg(any(feature = "default-themes", feature = "ansi-theme"))]
pub fn load_default_themes() {
    #[cfg(feature = "default-themes")]
    THEMES.set(RwLock::new(ThemeSet::load_defaults())).unwrap();
    #[cfg(feature = "ansi-theme")]
    {
        let themes: ThemeSet =
            syntect::dumps::from_binary(include_bytes!("../dumps/themes.themedump"));
        ANSI_THEME
            .set(themes.themes.get("ansi").unwrap().clone())
            .unwrap();
    }
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

pub fn find_syntax_by_name(name: &str) -> SyntaxReference {
    let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
    syntaxes.find_syntax_by_name(name).unwrap().clone()
}

pub fn find_syntax_by_extension(extension: &str) -> SyntaxReference {
    let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
    syntaxes
        .find_syntax_by_extension(extension)
        .unwrap()
        .clone()
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

#[derive(Clone, Copy)]
pub enum OverrideBackground {
    Empty,
    Color(ratatui::style::Color),
}

pub struct CodeHighlighter {
    theme: Theme,
    is_ansi_theme: bool,
    override_background: Option<OverrideBackground>,
}

impl CodeHighlighter {
    pub fn new_ansi() -> Self {
        let theme = ANSI_THEME.get().unwrap().clone();

        Self {
            theme,
            is_ansi_theme: true,
            override_background: None,
        }
    }

    pub fn new(theme: &str) -> Self {
        let themes = THEMES.get().unwrap().read().unwrap();
        let theme = themes.themes.get(theme).unwrap().clone();

        Self {
            theme,
            is_ansi_theme: false,
            override_background: None,
        }
    }

    pub fn override_background(mut self, background: OverrideBackground) -> Self {
        self.override_background = Some(background);
        self
    }

    pub fn highlight_file(&self, file: impl AsRef<Path>) -> Text {
        let mut highlighter = HighlightFile::new(
            file,
            &SYNTAXES
                .get()
                .expect("Syntaxes weren't initialized")
                .read()
                .unwrap(),
            &self.theme,
        )
        .unwrap();

        let mut line = String::new();
        let mut formatted = Vec::new();
        while highlighter.reader.read_line(&mut line).unwrap() > 0 {
            let highlighted = self
                .highlight_line(line.clone(), &mut highlighter.highlight_lines)
                .unwrap();
            formatted.push(highlighted);
            line.clear();
        }
        Text::from_iter(formatted.into_iter().flatten())
    }

    pub fn highlight_lines(
        &self,
        source: impl IntoLines,
        syntax: Option<SyntaxReference>,
    ) -> Text<'_> {
        let lines = source.into_lines();
        let syntax = match syntax {
            Some(syntax) => syntax,
            None => {
                let syntaxes = SYNTAXES.get().unwrap().read().unwrap();
                syntaxes
                    .find_syntax_by_first_line(&lines[0])
                    .unwrap()
                    .clone()
            }
        };
        let mut highlighter = HighlightLines::new(&syntax, &self.theme);
        let formatted = lines
            .into_iter()
            .map(|line| self.highlight_line(line, &mut highlighter))
            .collect::<Result<Vec<_>, syntect::Error>>();
        Text::from_iter(formatted.unwrap().into_iter().flatten())
    }

    fn highlight_line(
        &self,
        line: String,
        highlighter: &mut HighlightLines,
    ) -> Result<Vec<Line<'_>>, syntect::Error> {
        let line = if line.ends_with('\n') {
            line
        } else {
            format!("{line}\n")
        };

        let regions = highlighter.highlight_line(
            &line,
            &SYNTAXES
                .get()
                .expect("Syntaxes weren't initialized")
                .read()
                .unwrap(),
        )?;

        Ok(if self.is_ansi_theme {
            to_tui_text_ansi_theme(&regions[..], self.override_background)
        } else {
            to_tui_text(&regions, self.override_background)
        })
    }
}

#[cfg(feature = "ansi-theme")]
fn to_tui_text_ansi_theme(
    v: &[(syntect::highlighting::Style, &str)],
    background: Option<OverrideBackground>,
) -> Vec<Line<'static>> {
    let mut spans = vec![];
    let mut lines = vec![];
    for &(ref style, mut text) in v.iter() {
        let ends_with_newline = text.ends_with('\n');
        if ends_with_newline {
            text = &text[..text.len() - 1];
        }
        let text = text.to_string();

        let fg = if style.foreground.a == 0 {
            ansi_color_to_tui(style.foreground.r)
        } else if style.foreground.a == 1 {
            ratatui::style::Color::default()
        } else {
            ratatui::style::Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
        };
        let mut tui_style = ratatui::style::Style::new().fg(fg);
        if let Some(OverrideBackground::Color(bg)) = background {
            tui_style = tui_style.bg(bg);
        }
        tui_style = tui_style.add_modifier(syntect_modifiers_to_tui(&style.font_style));
        spans.push(Span::styled(text, tui_style));
        if ends_with_newline {
            lines.push(Line::from_iter(spans.drain(..)));
        }
    }
    lines
}

fn to_tui_text(
    v: &[(syntect::highlighting::Style, &str)],
    background: Option<OverrideBackground>,
) -> Vec<Line<'static>> {
    let mut spans = vec![];
    let mut lines = vec![];
    for &(ref style, mut text) in v.iter() {
        let ends_with_newline = text.ends_with('\n');
        if ends_with_newline {
            text = &text[..text.len() - 1];
        }
        let text = text.to_string();

        let tui_style = syntect_style_to_tui(style, background);

        spans.push(Span::styled(text, tui_style));

        if ends_with_newline {
            lines.push(Line::from_iter(spans.drain(..)));
        }
    }

    lines
}

fn syntect_style_to_tui(
    style: &syntect::highlighting::Style,
    override_background: Option<OverrideBackground>,
) -> ratatui::style::Style {
    let mut tui_style = ratatui::style::Style::new()
        .fg(ratatui::style::Color::Rgb(
            style.foreground.r,
            style.foreground.b,
            style.foreground.g,
        ))
        .add_modifier(syntect_modifiers_to_tui(&style.font_style));
    if let Some(bg) = override_background {
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
        _ => ratatui::style::Color::White,
    }
}
