use criterion::{black_box, criterion_group, criterion_main, Criterion};
use font_map::font::Font;

const SMALL_FONT: &[u8] = include_bytes!("../examples/slick.ttf");
const BEEG_FONT: &[u8] = include_bytes!("../google_material_symbols/font.ttf");

fn load(data: &[u8]) -> Font {
    Font::new(data).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("load_small_font", |b| {
        b.iter(|| load(black_box(SMALL_FONT)))
    });
    c.bench_function("load_large_font", |b| b.iter(|| load(black_box(BEEG_FONT))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
