//! # Google Material Symbols Font
//!
//! [![Crates.io](https://img.shields.io/crates/v/nerd_font_symbols.svg)](https://crates.io/crates/nerd_font_symbols/)
//! [![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
//! [![docs.rs](https://img.shields.io/docsrs/nerd_font_symbols)](https://docs.rs/nerd_font_symbols/latest/)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/nerd_font_symbols/master/LICENSE)
//!
//! This crate provides an enum of all the glyphs in the Google Material Symbols font.  
//! Additionally, it provides a way to load the font, and QOL features for using the font in iced.
//!
//! See <https://www.nerdfonts.com/> for more information
//!
//! **I am not affiliated with Google Inc., nor do I have any rights to the Google Material Symbols font.**  
//! This crate is published with a copy of the font, and its license, as allowed by the license.
//!
//! See [`Icon`] for the list of available icons, including their names, codepoints and a preview image.  
//! See [`Icon::FONT_FAMILY`] for the functions and constants available on the enum (So you don't need to scroll past 3,589 icons to find it!)
//!
//! ## Example
//!
//! ```rust
//! use nerd_font_symbols::{Icon, load_font};
//!
//! //
//! // You can access the icon by name, and get the postfix name, or codepoint
//! // You can also hover over the icon to see information about it, and a preview of the icon (as inline svg)
//! assert_eq!(Icon::FaArrowLeft.name(), "fa-arrow_left");
//! let codepoint = Icon::FaArrowLeft as u32;
//!
//! //
//! // You can also search for glyphs, and extract data about the font
//! let font = load_font();
//! let icon = font.glyph_named("fa-arrow_left").unwrap();
//! let svg = icon.svg_preview(); // The same as the inline svg in the hover
//! ```
//!
//! If you use iced there are some QOL features built-in:
//!
//! ```ignore
//! // `nerd_font_symbols::FONT_BYTES` is the raw bytes of the font, for loading into iced
//! // You need to activate the `iced` feature to use these features
//! let text_widget = Icon::FaArrowLeft.into_text(24); // A text widget with the icon, in the font, size 24
//! let widget: iced::Element<_> = Icon::FaArrowLeft.into(); // A text widget with the icon, in the default font size
//!
//! ```
//!
//! ## Features
//!
//! #### `svg-preview`
//! Default: On  
//! Provides a preview of the icon in the hover documentation, as an inline SVG.
//!
//! #### `iced`
//! Default: Off  
//! Provides some QOL features for using the font in iced, including a font definition, and conversion to an iced Text widget.
//!
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Re-export of the `font_map` crate, which provides a simple API for analyzing font files
pub use font_map;

//
// Generated font bindings
include!(env!("FONT_ENUM"));

/// The contents of the Google Material Icons font file
pub const FONT_BYTES: &[u8] = include_bytes!("../font.ttf");

/// Load the Google Material Icons font, returning a `font_map::Font` instance
#[allow(
    clippy::missing_panics_doc,
    reason = "The panic message is clear enough"
)]
#[must_use]
pub fn load_font() -> font_map::font::Font {
    font_map::font::Font::new(FONT_BYTES).expect("Bundled font was invalid!")
}

impl Icon {
    /// Returns a font definition for this font
    #[cfg(feature = "iced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
    #[must_use]
    pub fn iced_font() -> iced::Font {
        iced::font::Font {
            family: iced::font::Family::Name(Icon::FONT_FAMILY),
            ..Default::default()
        }
    }

    /// Converts this enum into an iced Text widget
    #[cfg(feature = "iced")]
    #[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
    #[must_use]
    pub fn into_text<'a, Theme>(
        self,
        font_size: impl Into<iced::Pixels>,
    ) -> iced::widget::Text<'a, Theme>
    where
        Theme: iced::widget::text::Catalog,
    {
        iced::widget::Text::new(char::from(self))
            .font(Self::iced_font())
            .size(font_size)
    }
}

#[cfg(feature = "iced")]
#[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
impl<'a, Message> From<Icon> for iced::Element<'a, Message> {
    fn from(value: Icon) -> Self {
        let font_size = iced::settings::Settings::default().default_text_size;
        value.into_text(font_size).into()
    }
}
