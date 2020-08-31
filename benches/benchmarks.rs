use criterion::{BenchmarkId, criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::Read;

use pico_detect::{Detector, Localizer, CascadeParameters, create_xorshift_rng};
use pico_detect::test_utils::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut fp = File::open("./models/puploc.bin").unwrap();
    let mut data = Vec::with_capacity(1200 * 1024);
    fp.read_to_end(&mut data).unwrap();

    c.bench_function("Localizer::from_readable", |b| {
        b.iter(|| Localizer::from_readable(data.as_slice()).unwrap())
    });

    let puploc = load_puploc_model();
    let (image, (left_pupil, _)) = load_test_image();
    let init_point = create_init_point(&left_pupil);

    c.bench_function("Localizer.localize", |b| {
        b.iter(|| puploc.localize(&image, &init_point))
    });

    let mut rng = create_xorshift_rng(42u64);

    let mut group = c.benchmark_group("Localizer.perturb_localize");
    for nperturbs in [15, 23, 31].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(nperturbs), nperturbs, |b, &nperturbs| {
            b.iter(|| puploc.perturb_localize(&image, &init_point, &mut rng, nperturbs))
        });
    }
    group.finish();

    let mut fp = File::open("./models/facefinder").unwrap();
    let mut data = Vec::with_capacity(235 * 1024);
    fp.read_to_end(&mut data).unwrap();

    c.bench_function("Detector::from_readable", |b| {
        b.iter(|| Detector::from_readable(data.as_slice()).unwrap())
    });

    let facefinder = load_facefinder_model();

    let params = CascadeParameters::new(100, 500, 0.1, 1.1);
    c.bench_function("Detector::run_cascade", |b| {
        b.iter(|| facefinder.run_cascade(&image, &params));
    });
    // let detections = facefinder.find_clusters(&image, &params, 0.1);

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
