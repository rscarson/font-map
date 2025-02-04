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
//! See <https://www.nerdfonts.com/> for more information
//!
//! **I am not affiliated with Nerd Fonts, nor do I have any rights to the `JetbrainsMono Nerd Font`.**  
//! This crate is published with a copy of the font, and its license, as allowed by the license.
//!
//! See [`Symbols`] for the list of available icons, including their names, codepoints and a preview image.  
//!
//! ## Example
//!
//! ```rust
//! use nerd_font::{Icon, load_font};
//!
//! //
//! // You can access the icon by name, and get the postfix name, or codepoint
//! // You can also hover over the icon to see information about it, and a preview of the icon (as inline svg)
//! assert_eq!(Symbols::, "fa-arrow_left");
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
//! // `nerd_font::FONT_BYTES` is the raw bytes of the font, for loading into iced
//! // You need to activate the `iced` feature to use these features
//! let text_widget = Symbols::dev::Android.into_text(24); // A text widget with the icon, in the font, size 24
//! let widget = Symbols::dev::Android.into_element(); // A text widget with the icon, in the default font size
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

/// The contents of the Font file
pub const FONT_BYTES: &[u8] = include_bytes!("../font.ttf");

/// Load the Nerd Font symbols, returning a `font_map::Font` instance
#[allow(
    clippy::missing_panics_doc,
    reason = "The panic message is clear enough"
)]
#[must_use]
pub fn load_font() -> font_map::font::Font {
    Symbols::dev::Android;
    font_map::font::Font::new(FONT_BYTES).expect("Bundled font was invalid!")
}

#[cfg(feature = "iced")]
#[cfg_attr(docsrs, doc(cfg(feature = "iced")))]
pub trait IcedExt {
    /// Returns an iced font definition for this font
    fn iced_font() -> iced::Font;

    /// Converts this enum into an iced Text widget
    #[must_use]
    fn into_text<'a, Theme>(
        self,
        font_size: impl Into<iced::Pixels>,
    ) -> iced::widget::Text<'a, Theme>
    where
        Theme: iced::widget::text::Catalog;

    /// Converts this enum into an iced Element with the default font size
    fn into_element<'a, Message>(self) -> iced::Element<'a, Message>
    where
        Message: 'a;
}
#[cfg(feature = "iced")]
impl IcedExt for font_map::font::Glyph {
    fn iced_font() -> iced::Font {
        iced::font::Font {
            family: iced::font::Family::Name(Symbols::FONT_FAMILY),
            ..Default::default()
        }
    }

    #[must_use]
    fn into_text<'a, Theme>(
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

    fn into_element<'a, Message>(self) -> iced::Element<'a, Message>
    where
        Message: 'a,
    {
        let font_size = iced::settings::Settings::default().default_text_size;
        self.into_text(font_size).into()
    }
}
