<!-- cargo-rdme start -->

# Google Material Symbols Font

[![Crates.io](https://img.shields.io/crates/v/google_material_symbols.svg)](https://crates.io/crates/google_material_symbols/)
[![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
[![docs.rs](https://img.shields.io/docsrs/google_material_symbols)](https://docs.rs/google_material_symbols/latest/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/google_material_symbols/master/LICENSE)

This crate provides an enum of all the glyphs in the Google Material Symbols font.  
Additionally, it provides a way to load the font, and QOL features for using the font in iced.

In addition - you can hover over the icons in your IDE to see a preview of the icon!

See <https://fonts.google.com/icons> for more information

**I am not affiliated with Google Inc., nor do I have any rights to the Google Material Symbols font.**  
This crate is published with a copy of the font, and its license, as allowed by the license.

See [`GoogleMaterialSymbols`] for the list of available icons, including their names, codepoints and a preview image.  
See [`GoogleMaterialSymbols::FONT_FAMILY`] for the functions and constants available on the enum (So you don't need to scroll past 3,589 icons to find it!)

-----

The individual glyphs are in the `GoogleMaterialSymbols` enum:

```rust
use google_material_symbols::GoogleMaterialSymbols;
let _ = GoogleMaterialSymbols::MagicButton;
```

-----

Each glyph contains the following information:
- Unicode codepoint: e.g. `GoogleMaterialSymbols::MagicButton as u32`
- Postfix name: e.g. `GoogleMaterialSymbols::MagicButton.name()`
- Glyph preview image, visible in the documentation, and by hovering over the glyphs in your IDE!

You can also get the actual char from the enum, with `char::from(GoogleMaterialSymbols::MagicButton)`, or `GoogleMaterialSymbols::MagicButton.to_string()`

-----

If you use `iced` there are some QOL features built-in:  
**NOTE: ** you will need to activate the `iced` crate-level feature to use these!

- [`FONT_BYTES`] is the raw bytes of the font, for loading into iced
- [`IcedExt`] provides the helper functions for using the font in iced
- Glyphs also implement `Into<iced::Element>`, which will use the default font size

```rust
use google_material_symbols::{IcedExt, categories::Dev};

// A text widget configured to use the icon font, with the selected glyph, and a font size of 24
let text_widget = Dev::Android.into_text(24);
```

You will additionally need to load the font, by calling `.font(google_material_symbols::FONT_BYTES)` on your `iced::Application`.

## Crate Features

#### `iced`
Default: Off  
Provides some QOL features for using the font in iced, including a font definition, and conversion to an iced Text widget.

<!-- cargo-rdme end -->
