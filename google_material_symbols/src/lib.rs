//! # Google Material Symbols Font
//!
//! [![Crates.io](https://img.shields.io/crates/v/google_material_symbols.svg)](https://crates.io/crates/google_material_symbols/)
//! [![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
//! [![docs.rs](https://img.shields.io/docsrs/google_material_symbols)](https://docs.rs/google_material_symbols/latest/)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/google_material_symbols/master/LICENSE)
//!
//! This crate provides an enum of all the glyphs in the Google Material Symbols font.  
//! Additionally, it provides a way to load the font, and QOL features for using the font in iced.
//!
//! In addition - you can hover over the icons in your IDE to see a preview of the icon!
//!
//! See <https://fonts.google.com/icons> for more information
//!
//! **I am not affiliated with Google Inc., nor do I have any rights to the Google Material Symbols font.**  
//! This crate is published with a copy of the font, and its license, as allowed by the license.
//!
//! See [`GoogleMaterialSymbols`] for the list of available icons, including their names, codepoints and a preview image.  
//! See [`GoogleMaterialSymbols::FONT_FAMILY`] for the functions and constants available on the enum (So you don't need to scroll past 3,589 icons to find it!)
//!
//! -----
//!
//! The individual glyphs are in the `GoogleMaterialSymbols` enum:
//!
//! ```rust
//! use google_material_symbols::GoogleMaterialSymbols;
//! let _ = GoogleMaterialSymbols::MagicButton;
//! ```
//!
//! -----
//!
//! Each glyph contains the following information:
//! - Unicode codepoint: e.g. `GoogleMaterialSymbols::MagicButton as u32`
//! - Postfix name: e.g. `GoogleMaterialSymbols::MagicButton.name()`
//! - Glyph preview image, visible in the documentation, and by hovering over the glyphs in your IDE!
//!
//! You can also get the actual char from the enum, with `char::from(GoogleMaterialSymbols::MagicButton)`, or `GoogleMaterialSymbols::MagicButton.to_string()`
//!
//! -----
//!
//! If you use `iced` there are some QOL features built-in:  
//! **NOTE: ** you will need to activate the `iced` crate-level feature to use these!
//!
//! - [`FONT_BYTES`] is the raw bytes of the font, for loading into iced
//! - [`IcedExt`] provides the helper functions for using the font in iced
//! - Glyphs also implement `Into<iced::Element>`, which will use the default font size
//!
//! ```ignore
//! use google_material_symbols::{IcedExt, categories::Dev};
//!
//! // A text widget configured to use the icon font, with the selected glyph, and a font size of 24
//! let text_widget = Dev::Android.into_text(24);
//! ```
//!
//! You will additionally need to load the font, by calling `.font(google_material_symbols::FONT_BYTES)` on your `iced::Application`.
//!
//! ## Crate Features
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

font_map::include_font!(GoogleMaterialSymbols);

/// Extension trait for using these icons from within iced
///
/// - [`FONT_BYTES`] is the raw bytes of the font, for loading into iced
/// - `GoogleMaterialSymbols` also implements `Into<iced::Element>`, which will use the default font size
///
/// ```rust
/// use google_material_symbols::{IcedExt, GoogleMaterialSymbols};
///
/// // A text widget configured to use the icon font, with the selected glyph, and a font size of 24
/// let text_widget = GoogleMaterialSymbols.into_text(24);
/// ```
///
/// You will additionally need to load the font, by calling `.font(google_material_symbols::FONT_BYTES)` on your `iced::Application`.
#[cfg(feature = "iced")]
#[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
pub trait IcedExt {
    /// Returns a font definition for this font  
    /// Used for the `font` method on iced text widgets
    #[must_use]
    fn iced_font() -> iced::Font;

    /// Converts this enum into an iced Text widget  
    /// Sets the font-size of the new widget
    #[must_use]
    fn into_text<'a, Theme>(
        self,
        font_size: impl Into<iced::Pixels>,
    ) -> iced::widget::Text<'a, Theme>
    where
        Theme: iced::widget::text::Catalog;
}

#[cfg(feature = "iced")]
#[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
impl<S: Into<GoogleMaterialSymbols>> IcedExt for S {
    fn iced_font() -> iced::Font {
        iced::font::Font {
            family: iced::font::Family::Name(GoogleMaterialSymbols::FONT_FAMILY),
            ..Default::default()
        }
    }

    fn into_text<'a, Theme>(
        self,
        font_size: impl Into<iced::Pixels>,
    ) -> iced::widget::Text<'a, Theme>
    where
        Theme: iced::widget::text::Catalog,
    {
        iced::widget::Text::new(char::from(Into::<GoogleMaterialSymbols>::into(self)))
            .font(Self::iced_font())
            .size(font_size)
    }
}

#[cfg(feature = "iced")]
#[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
impl<'a, Message> From<GoogleMaterialSymbols> for iced::Element<'a, Message> {
    fn from(value: GoogleMaterialSymbols) -> Self {
        let font_size = iced::settings::Settings::default().default_text_size;
        value.into_text(font_size).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test() {
        let font = load_font();
        assert!(!font.glyphs().is_empty());
        assert!(!GoogleMaterialSymbols::FONT_BYTES.is_empty());
        let _ = GoogleMaterialSymbols::MagicButton;
    }
}
