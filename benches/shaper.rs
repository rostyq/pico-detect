use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::GrayImage;
use nalgebra::Point3;
use pico_detect::Shaper;

fn criterion_benchmark(c: &mut Criterion) {
    let data = include_bytes!("../models/shaper_5_face_landmarks.bin").to_vec();

    c.bench_function("Shaper::from_readable", |b| {
        b.iter(|| Shaper::from_readable(black_box(data.as_slice())).unwrap())
    });

    let shaper = Shaper::from_readable(data.as_slice()).unwrap();
    let image = GrayImage::new(640, 480);
    let point = Point3::new(200., 200., 100.);

    c.bench_function("Shaper.predict", |b| {
        b.iter(|| shaper.predict(black_box(&image), black_box(&point)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
