use std::path::Path;

use font_map_core::{font::Font, FontConst};

const FONT_BYTES: &[u8] = include_bytes!("font.ttf");

fn main() {
    println!("cargo:rerun-if-changed=font.ttf");

    let font = Font::new(FONT_BYTES).expect("Bundled font was invalid!");
    let code = FontConst::from_font("Symbols", &font).codegen().to_string();

    //
    // Create the target file
    let dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let target = Path::new(&dir).join("font_generated.rs");

    //
    // Provide an ENV var with the path to the generated file
    println!("cargo:rustc-env=FONT_ENUM={}", target.display());

    //
    // Write the file
    std::fs::write(target, code).expect("Failed to write generated icon-enum");
}
