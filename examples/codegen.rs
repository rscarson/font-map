use font_map::codegen::FontDesc;
use font_map::error::ParseResult;
use font_map::font::Font;

fn main() -> ParseResult<()> {
    //
    // Load the font, and create a code generator from it
    let font = Font::from_file("nerd_font/font.ttf")?;
    let generator = FontDesc::from_font("Icon", &font, false);

    //
    // We can optionally inject extra code into the generated enum's impl block
    let code_tokens = generator.codegen(None);
    println!("Generated some code: {}", !code_tokens.is_empty());
    Ok(())
}
