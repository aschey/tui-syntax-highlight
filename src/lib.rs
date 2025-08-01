mod highlighted_text;
mod highlighter;

use std::borrow::Cow;

pub use highlighted_text::*;
pub use highlighter::*;
pub use syntect;

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
