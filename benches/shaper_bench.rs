#[path = "./common/macros.rs"]
mod macros;

use std::fs;

use criterion::{black_box, criterion_group, Criterion};

use image;
use pico_detect::{Shaper, utils::Square};

fn bench_shaper_load(c: &mut Criterion) {
    let model_data = fs::read(model_path!(shaper)).unwrap();

    c.bench_function("Shaper::load", |b| {
        b.iter(|| Shaper::load(black_box(model_data.as_slice())).unwrap())
    });
}

fn bench_shaper_shape(c: &mut Criterion) {
    let image = load_test_image!();
    let shaper = load_model!(shaper);

    let r = Square::new(213, 225, 153).into();

    c.bench_function("Shaper::shape", |b| {
        b.iter(|| shaper.shape(black_box(&image), black_box(r)));
    });
}

criterion_group!(benches, bench_shaper_load, bench_shaper_shape);
