//! A library for working with the Google Material Icons font
//!
//! Provides access to the Sharp/Regular Google Material Icons font, as well as a code-generated enum of all the glyphs in the font.
//!
//! **I am not affiliated with Google Inc., nor do I have any rights to the Google Material Symbols font.**  
//! This crate is published with a copy of the font, and its license, as allowed by the license.
//!

/// Re-export the `font_map` crate, which provides a simple API for analyzing font files
pub use font_map;

//
// Generated font bindings
include!(env!("FONT_ENUM"));

/// The contents of the Google Material Icons font file
pub const FONT_BYTES: &[u8] = include_bytes!("../font.ttf");

/// Load the Google Material Icons font, returning a `font_map::Font` instance
pub fn load_font() -> font_map::font::Font {
    font_map::font::Font::new(FONT_BYTES).expect("Bundled font was invalid!")
}

#[cfg(feature = "iced")]
impl Icon {
    /// Returns a font definition for this font
    pub fn iced_font() -> iced::Font {
        iced::font::Font {
            family: iced::font::Family::Name(Icon::FONT_FAMILY),
            ..Default::default()
        }
    }

    /// Converts this enum into an iced Text widget
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
impl<'a, Message> From<Icon> for iced::Element<'a, Message> {
    fn from(value: Icon) -> Self {
        let font_size = iced::settings::Settings::default().default_text_size;
        value.into_text(font_size).into()
    }
}
