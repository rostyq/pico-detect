#[macro_use]
mod common;

use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use image;
use pico_detect::{Localizer, utils::Square};

fn bench_localizer_load(c: &mut Criterion) {
    let model_data = fs::read(model_path!(puploc)).unwrap();

    c.bench_function("Localizer::load", |b| {
        b.iter(|| Localizer::load(black_box(model_data.as_slice())).unwrap())
    });
}

fn bench_localizer_localize(c: &mut Criterion) {
    let image = load_test_image!();
    let localizer = load_model!(puploc);

    let s = Square::new(310, 247, 38).into();

    c.bench_function("Localizer::localize", |b| {
        b.iter(|| localizer.localize(black_box(&image), black_box(s)));
    });
}

criterion_group!(benches, bench_localizer_load, bench_localizer_localize);
criterion_main!(benches);
