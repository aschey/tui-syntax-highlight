//#![warn(missing_docs)]
#![forbid(clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod convert;
mod highlighter;

use std::fmt::Display;
use std::io;

pub use convert::*;
pub use highlighter::*;
pub use syntect;
#[cfg(feature = "profile")]
pub use termprofile;

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
