//! A library for working with the Google Material Icons font
//!
//! Provides access to the Sharp/Regular Google Material Icons font, as well as a code-generated enum of all the glyphs in the font.

/// Re-export the `font_map` crate, which provides a simple API for analyzing font files
pub use font_map;

mod font_generated;
pub use font_generated::*;

/// The contents of the Google Material Icons font file
pub const FONT_BYTES: &[u8] = include_bytes!("../font.ttf");

/// Load the Google Material Icons font, returning a `font_map::Font` instance
pub fn load_font() -> font_map::font::Font {
    font_map::font::Font::new(FONT_BYTES).expect("Bundled font was invalid!")
}
