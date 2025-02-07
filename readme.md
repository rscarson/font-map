<!-- cargo-rdme start -->

# font-map
## Font parser / enumerator with support for code generation

[![Crates.io](https://img.shields.io/crates/v/font-map.svg)](https://crates.io/crates/font-map/)
[![Build Status](https://github.com/rscarson/font-map/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/rscarson/font-map/actions?query=branch%3Amaster)
[![docs.rs](https://img.shields.io/docsrs/font-map)](https://docs.rs/font-map/latest/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/font-map/master/LICENSE)

This crate provides functionality for parsing font files and enumerating the glyphs they contain.

The base usecase for this crate is to create an enum of all the glyphs in a font file,  
for use in fontend projects, where you want to refer to glyphs by name rather than by codepoint:

```rust
use font_map::font;

font!(Icon, "google_material_symbols/font.ttf");

const DELETE: Icon = Icon::Delete;
```

The generated code includes information for each glyph, such as:
- codepoint, and postfix-name
- Plus a generated SVG preview image visible on hover

You can also access `Icon::FONT_FAMILY` to simplify font usage in your frontend.

-----

Another use is to use it for introspection of font files:

```rust
use font_map::font::Font;

let font = Font::from_file("google_material_symbols/font.ttf")?;
if let Some(glyph) = font.glyph_named("delete") {
    let codepoint = glyph.codepoint();
    let svg = glyph.svg_preview();
}
```

## Features
- `macros` - Enables the `font!` macro for code generation
- `codegen` - Enables the `FontCodegenExt` trait for runtime code generation
- `extended-svg` - Enables compressed and base64 encoded SVG data in the generated code (Needed for image previews)

## Known Limitations
This crate was made for a very specific use-case, and as such currently has a few limitations:
- Only supports TTF fonts
- And even then, only a subset of the spec, namely:
- Only some formats of the `cmap` table
- Only Unicode, or MS encoding 1 and 10, and `Macintosh::0` of the `name` table
- Only formats 2.5 or below of the `post` table

<!-- cargo-rdme end -->
