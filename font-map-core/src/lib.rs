//! Core functionality for `font-map`
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
mod reader;

#[cfg(feature = "codegen")]
mod codegen;

mod font;
mod unicode_range;

#[cfg(feature = "codegen")]
pub use codegen::FontCodegenExt;

pub use font::*;

/// This module contains the raw data structures from parsing font files
pub mod raw {
    pub mod ttf;
}
