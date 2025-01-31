<!-- cargo-rdme start -->

# Google Material Symbols Font

[![Crates.io](https://img.shields.io/crates/v/google_material_symbols.svg)](https://crates.io/crates/google_material_symbols/)
[![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
[![docs.rs](https://img.shields.io/docsrs/google_material_symbols)](https://docs.rs/google_material_symbols/latest/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/google_material_symbols/master/LICENSE)

This crate provides an enum of all the glyphs in the Google Material Symbols font.  
Additionally, it provides a way to load the font, and QOL features for using the font in iced.

**I am not affiliated with Google Inc., nor do I have any rights to the Google Material Symbols font.**  
This crate is published with a copy of the font, and its license, as allowed by the license.

See [`Icon`] for the list of available icons, including their names, codepoints and a preview image.

```rust
use google_material_symbols::{Icon, load_font};

//
// You can access the icon by name, and get the postfix name, or codepoint
// You can also hover over the icon to see information about it, and a preview of the icon (as inline svg)
assert_eq!(Icon::Delete.name(), "delete");
let codepoint = Icon::Delete as u32;

//
// You can also search for glyphs, and extract data about the font
let font = load_font();
let icon = font.glyph_named("delete").unwrap();
let svg = icon.svg_outline(); // The same as the inline svg in the hover

//
// If you use iced there are some QOL features built-in
// `google_material_symbols::FONT_BYTES` is the raw bytes of the font, for loading into iced
// You need to activate the `iced` feature to use these features
let text_widget = Icon::Delete.into_text(24); // A text widget with the icon, in the font, size 24
let widget: iced::Element<_> = Icon::Delete.into(); // A text widget with the icon, in the default font size

```

<!-- cargo-rdme end -->
