//! # `JetbrainsMono Nerd Font`
//!
//! [![Crates.io](https://img.shields.io/crates/v/nerd_font.svg)](https://crates.io/crates/nerd_font/)
//! [![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
//! [![docs.rs](https://img.shields.io/docsrs/nerd_font)](https://docs.rs/nerd_font/latest/)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/nerd_font/master/LICENSE)
//!
//! This crate provides an enum of all the glyphs in the `JetbrainsMono Nerd Font`.  
//! Additionally, it provides a way to load the font, and QOL features for using the font in iced.
//!
//! In addition - you can hover over the icons in your IDE to see a preview of the icon!
//!
//! See <https://www.nerdfonts.com/> for more information
//!
//! **I am not affiliated with Nerd Fonts, nor do I have any rights to the `JetbrainsMono Nerd Font`.**  
//! This crate is published with a copy of the font, and its license, as allowed by the license.
//!
//! See [`NerdFont`] or [`categories`] for the list of available icons, including their names, codepoints and a preview image.  
//!
//! -----
//!
//! The individual glyphs are in seperate enums inside of the [`categories`] module:
//! ```rust
//! use nerd_font::categories;
//!
//! let _ = categories::Dev::Android;
//! let _ = categories::Fa::ArrowLeft;
//! ```
//!
//! There is also an enum encapsulating all the glyphs, [`NerdFont`], which can be converted to and from the individual enums:
//! ```rust
//! use nerd_font::{NerdFont, categories::Dev};
//! let _: NerdFont = Dev::Android.into();
//! ```
//!
//! -----
//!
//! Each glyph contains the following information:
//! - Unicode codepoint: e.g. `Dev::Android as u32`
//! - Postfix name: e.g. `Dev::Android.name()`
//! - Glyph preview image, visible in the documentation, and by hovering over the glyphs in your IDE!
//!
//! You can also get the actual char from the enum, with `char::from(Dev::Android)`, or `Dev::Android.to_string()`
//!
//! -----
//!
//! If you use `iced` there are some QOL features built-in:  
//! **NOTE: ** you will need to activate the `iced` crate-level feature to use these!
//!
//! - [`NerdFont::FONT_BYTES`] is the raw bytes of the font, for loading into iced
//! - [`IcedExt`] provides the helper functions for using the font in iced
//! - `NerdFont` also implements `Into<iced::Element>`, which will use the default font size
//!
//! ```ignore
//! use nerd_font::{IcedExt, categories::Dev};
//!
//! // A text widget configured to use the icon font, with the selected glyph, and a font size of 24
//! let text_widget = Dev::Android.into_text(24);
//! ```
//!
//! You will additionally need to load the font, by calling `.font(NerdFont::FONT_BYTES)` on your `iced::Application`.
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

font_map::include_font!(NerdFont);

/// Extension trait for using these icons from within iced
///
/// - [`NerdFont::FONT_BYTES`] is the raw bytes of the font, for loading into iced
/// - `NerdFont` also implements `Into<iced::Element>`, which will use the default font size
///
/// ```rust
/// use nerd_font::{IcedExt, categories::Dev};
///
/// // A text widget configured to use the icon font, with the selected glyph, and a font size of 24
/// let text_widget = Dev::Android.into_text(24);
/// ```
///
/// You will additionally need to load the font, by calling `.font(NerdFont::FONT_BYTES)` on your `iced::Application`.
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
impl<S: Into<NerdFont>> IcedExt for S {
    fn iced_font() -> iced::Font {
        iced::font::Font {
            family: iced::font::Family::Name(NerdFont::FONT_FAMILY),
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
        iced::widget::Text::new(char::from(Into::<NerdFont>::into(self)))
            .font(Self::iced_font())
            .size(font_size)
    }
}

#[cfg(feature = "iced")]
#[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
impl<'a, Message> From<NerdFont> for iced::Element<'a, Message> {
    fn from(value: NerdFont) -> Self {
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
        assert!(!NerdFont::FONT_BYTES.is_empty());
        let _ = categories::Dev::Ansible;
    }
}
