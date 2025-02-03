use criterion::{black_box, criterion_group, criterion_main, Criterion};
use font_map::{font::Font, FontCodegenExt};

const FONT: &[u8] = include_bytes!("../google_material_symbols/font.ttf");

fn load(font: &Font) -> String {
    font.gen_enum("Icon").to_string()
}

fn criterion_benchmark(c: &mut Criterion) {
    let font = Font::new(FONT).unwrap();
    c.bench_function("codegen", |b| b.iter(|| load(black_box(&font))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
