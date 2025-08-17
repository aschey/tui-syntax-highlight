//#![warn(missing_docs)]
#![forbid(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod convert;
mod highlighter;

use std::borrow::Cow;
use std::fmt::Display;
use std::io;

pub use convert::*;
pub use highlighter::*;
pub use {syntect, termprofile};

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

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(e) => write!(f, "error reading from source: {e:?}"),
            Self::Highlight(e) => write!(f, "error highlighting content: {e:?}"),
        }
    }
}
