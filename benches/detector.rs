use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::GrayImage;
use pico_detect::{CascadeParameters, Detector};

fn criterion_benchmark(c: &mut Criterion) {
    let data = include_bytes!("../models/facefinder").to_vec();

    c.bench_function("Detector::from_readable", |b| {
        b.iter(|| Detector::from_readable(black_box(data.as_slice())).unwrap())
    });

    let facefinder = Detector::from_readable(data.as_slice()).unwrap();
    let image = GrayImage::new(640, 480);

    let params = CascadeParameters::new(100, 640, 0.05, 1.1);
    c.bench_function("Detector::run_cascade", |b| {
        b.iter(|| facefinder.run_cascade(black_box(&image), &params));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
