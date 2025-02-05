use criterion::{black_box, criterion_group, criterion_main, Criterion};
use font_map::{font::Font, FontDesc};

const GOOGLE_FONT: &[u8] = include_bytes!("../google_material_symbols/font.ttf");
const NERD_FONT: &[u8] = include_bytes!("../nerd_font/font.ttf");

fn generate_code(font: &Font, skip_categories: bool) -> String {
    let generator = FontDesc::from_font("Icon", font, skip_categories);
    generator.codegen(None).to_string()
}

fn criterion_benchmark(c: &mut Criterion) {
    let google_font = Font::new(GOOGLE_FONT).unwrap();
    let nerd_font = Font::new(NERD_FONT).unwrap();

    c.bench_function("codegen_singleton", |b| {
        b.iter(|| generate_code(black_box(&google_font), black_box(true)))
    });
    c.bench_function("codegen_categorized", |b| {
        b.iter(|| generate_code(black_box(&nerd_font), black_box(false)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
