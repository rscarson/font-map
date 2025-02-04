<!-- cargo-rdme start -->

# JetbrainsMono Nerd Font

[![Crates.io](https://img.shields.io/crates/v/nerd_font_symbols.svg)](https://crates.io/crates/nerd_font_symbols/)
[![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
[![docs.rs](https://img.shields.io/docsrs/nerd_font_symbols)](https://docs.rs/nerd_font_symbols/latest/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/nerd_font_symbols/master/LICENSE)

This crate provides an enum of all the glyphs in the JetbrainsMono Nerd Font.  
Additionally, it provides a way to load the font, and QOL features for using the font in iced.

See <https://www.nerdfonts.com/> for more information

**I am not affiliated with Nerd Fonts, nor do I have any rights to the JetbrainsMono Nerd Font.**  
This crate is published with a copy of the font, and its license, as allowed by the license.

See [`Icon`] for the list of available icons, including their names, codepoints and a preview image.  
See [`Icon::FONT_FAMILY`] for the functions and constants available on the enum (So you don't need to scroll past 3,589 icons to find it!)

## Example

```rust
use nerd_font_symbols::{Icon, load_font};

//
// You can access the icon by name, and get the postfix name, or codepoint
// You can also hover over the icon to see information about it, and a preview of the icon (as inline svg)
assert_eq!(Icon::FaArrowLeft.name(), "fa-arrow_left");
let codepoint = Icon::FaArrowLeft as u32;

//
// You can also search for glyphs, and extract data about the font
let font = load_font();
let icon = font.glyph_named("fa-arrow_left").unwrap();
let svg = icon.svg_preview(); // The same as the inline svg in the hover
```

If you use iced there are some QOL features built-in:

```rust
// `nerd_font_symbols::FONT_BYTES` is the raw bytes of the font, for loading into iced
// You need to activate the `iced` feature to use these features
let text_widget = Icon::FaArrowLeft.into_text(24); // A text widget with the icon, in the font, size 24
let widget: iced::Element<_> = Icon::FaArrowLeft.into(); // A text widget with the icon, in the default font size

```

## Features

#### `svg-preview`
Default: On  
Provides a preview of the icon in the hover documentation, as an inline SVG.

#### `iced`
Default: Off  
Provides some QOL features for using the font in iced, including a font definition, and conversion to an iced Text widget.

<!-- cargo-rdme end -->
