//#![warn(missing_docs)]
#![forbid(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod highlighter;

use std::borrow::Cow;
use std::fmt::Display;
use std::io;

pub use highlighter::*;
use ratatui::style::Color;
pub use syntect;
use termprofile::anstyle;

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

impl IntoLines for Vec<Cow<'_, str>> {
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

impl IntoLines for Cow<'_, str> {
    fn into_lines(self) -> Vec<String> {
        match self {
            Self::Owned(s) => s.into_lines(),
            Self::Borrowed(s) => s.into_lines(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Source(io::Error),
    Highlight(syntect::Error),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorMode {
    TrueColor,
    Ansi256,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(e) => write!(f, "error reading from source: {e:?}"),
            Self::Highlight(e) => write!(f, "error highlighting content: {e:?}"),
        }
    }
}

pub fn syntect_style_to_tui(
    style: syntect::highlighting::Style,
    color_mode: ColorMode,
) -> ratatui::style::Style {
    let mut tui_style = ratatui::style::Style::new();
    if let Some(fg) = syntect_color_to_tui(style.foreground, color_mode) {
        tui_style = tui_style.fg(fg);
    }
    if let Some(bg) = syntect_color_to_tui(style.background, color_mode) {
        tui_style = tui_style.bg(bg);
    }
    tui_style.add_modifier(syntect_modifiers_to_tui(&style.font_style))
}

pub fn syntect_color_to_tui(
    color: syntect::highlighting::Color,
    color_mode: ColorMode,
) -> Option<Color> {
    if color.a == 0 {
        Some(match color.r {
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
            c => ratatui::style::Color::Indexed(c),
        })
    } else if color.a == 1 {
        None
    } else if color_mode == ColorMode::TrueColor {
        Some(Color::Rgb(color.r, color.g, color.b))
    } else {
        Some(Color::Indexed(termprofile::rgb_to_ansi256(
            anstyle::RgbColor(color.r, color.g, color.b),
        )))
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
