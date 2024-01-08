#[path = "./common/macros.rs"]
mod macros;

use std::fs;

use criterion::{black_box, Criterion};

use image;
use pico_detect::{Detector, Square};

pub fn bench_load(c: &mut Criterion) {
    let model_data = fs::read(model_path!(facefinder)).unwrap();

    c.bench_function("Detector::load", |b| {
        b.iter(|| Detector::load(black_box(model_data.as_slice())).unwrap())
    });
}

pub fn bench_inference(c: &mut Criterion) {
    let image = load_test_image!();
    let detector = load_model!(facefinder);

    let s = Square::new(216, 208, 170);

    c.bench_function("Detector::classify[inference]", |b| {
        b.iter(|| detector.classify(black_box(&image), black_box(s)));
    });
}
