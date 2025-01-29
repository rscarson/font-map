//!
//! This example will demonstrate how to enumerate all the glyphs in a font as an enum  
//! For this example, we will generate it using the `font!` macro
//!
//! See the `google-material-symbols` crate for doing this in `build.rs` instead
//!
use font_map::font;

//
// A very simple font with only a few glyphs
font!(Font, "examples/slick.ttf");

fn main() {
    println!(
        "Font `{}` contains {} glyphs",
        Font::FONT_FAMILY,
        Font::GLYPHS
    );

    //
    // Hover over the variant to see a preview of the glyph!
    // (If rust-analyzer or equivalent is installed)
    let arrow_left = Font::Arrowleft;
    println!("Arrow left: {arrow_left}");
}
