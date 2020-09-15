use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::GrayImage;
use nalgebra::Point3;
use pico_detect::{create_xorshift_rng, Localizer};

fn criterion_benchmark(c: &mut Criterion) {
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
