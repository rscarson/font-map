//!
//! This example will demonstrate how to enumerate all the glyphs in a font as an enum  
//! For this example, we will generate it using the `font!` macro
//!
//! See the `google-material-symbols` crate for doing this in `build.rs` instead
//!
use font_map::font;

//
// A very simple font with only a few glyphs
font!(Symbols, "examples/slick.ttf");

fn main() {
    let num_glyphs = Symbols::TOTAL_GLYPHS;
    let font_family = Symbols::FONT_FAMILY;
    println!("`{font_family}` has {num_glyphs} glyphs");

    //
    // Hover over `Symbols::Bullet` to see the glyph!
    let symbol_name = Symbols::Bullet.name();
    let symbol_code = Symbols::Bullet as u32;
    let symbol_str = Symbols::Bullet.to_string();
    println!("The glyph `{symbol_name}` has codepoint {symbol_code:04x}: `{symbol_str}`");
}
