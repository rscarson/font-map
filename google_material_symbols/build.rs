use font_map::{Font, FontCodegenExt};

const FONT_BYTES: &[u8] = include_bytes!("font.ttf");

fn main() {
    println!("cargo:rerun-if-changed=font.ttf");

    let font = Font::new(FONT_BYTES).unwrap();
    let code = font.gen_enum("Icon").to_string();

    std::fs::write("src/font_generated.rs", code).unwrap();
}
