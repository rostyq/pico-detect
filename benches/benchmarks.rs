use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::GrayImage;
use nalgebra::Point3;
use pico_detect::{create_xorshift_rng, CascadeParameters, Detector, Localizer, Shaper};

fn benchmark_detector(c: &mut Criterion) {
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

fn benchmark_shaper(c: &mut Criterion) {
    let data = include_bytes!("../models/shaper_5_face_landmarks.bin").to_vec();

    c.bench_function("Shaper::from_readable", |b| {
        b.iter(|| Shaper::from_readable(black_box(data.as_slice())).unwrap())
    });

    let shaper = Shaper::from_readable(data.as_slice()).unwrap();
    let image = GrayImage::new(640, 480);
    let point = Point3::new(200., 200., 100.);

    let mut group = c.benchmark_group("shaper");
    group.warm_up_time(std::time::Duration::from_secs(10));

    group.bench_function("Shaper.predict", |b| {
        b.iter(|| shaper.predict(black_box(&image), black_box(&point)));
    });
    group.finish();
}

fn benchmark_localizer(c: &mut Criterion) {
    let data = include_bytes!("../models/puploc.bin").to_vec();

    c.bench_function("Localizer::from_readable", |b| {
        b.iter(|| Localizer::from_readable(black_box(data.as_slice())).unwrap())
    });

    let puploc = Localizer::from_readable(data.as_slice()).unwrap();
    let image = GrayImage::new(640, 480);
    let point = Point3::new(200., 200., 100.);

    c.bench_function("Localizer.localize", |b| {
        b.iter(|| puploc.localize(black_box(&image), black_box(&point)));
    });

    let mut rng = create_xorshift_rng(42u64);
    let mut group = c.benchmark_group("Localizer.perturb_localize");
    for nperturbs in [15, 23, 31].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(nperturbs),
            nperturbs,
            |b, &nperturbs| {
                b.iter(|| {
                    puploc.perturb_localize(
                        black_box(&image),
                        black_box(&point),
                        &mut rng,
                        nperturbs,
                    )
                })
            },
        );
    }
    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_detector(c);
    benchmark_shaper(c);
    benchmark_localizer(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
