//!
//! This example will demonstrate loading a font and viewing the glyphs in it
//!
use font_map::error::ParseResult;
use font_map::font::{Font, StringKind};

fn main() -> ParseResult<()> {
    let font = Font::from_file("google_material_symbols/font.ttf")?;
    let font_name = font.string(StringKind::FullFontName).unwrap();
    println!(
        "Font `{font_name}` contains {} glyphs: ",
        font.glyphs().len()
    );

    font.glyphs().iter().for_each(|glyph| {
        let name = glyph.name();
        let codepoint = glyph.codepoint();
        println!("- `{name}` has codepoint {codepoint:04x}");

        //
        // You can also export the glyph as an image
        let _outline = glyph.svg_outline();
    });

    Ok(())
}
