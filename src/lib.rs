#![warn(missing_docs, missing_debug_implementations)]
#![forbid(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod convert;
mod highlighter;

use std::fmt::{self, Display};
use std::io;

pub use convert::*;
pub use highlighter::*;
pub use syntect;
#[cfg(feature = "termprofile")]
pub use termprofile;

/// Error returned from the syntax highlighter.
#[derive(Debug)]
pub enum Error {
    /// Error reading from source.
    Read(io::Error),
    /// Error highlighting content.
    Highlight(syntect::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(e) => write!(f, "error reading from source: {e:?}"),
            Self::Highlight(e) => write!(f, "error highlighting content: {e:?}"),
        }
    }
}
