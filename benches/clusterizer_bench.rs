#[path = "./common/macros.rs"]
mod macros;

use rand::distributions::OpenClosed01;
use rand::prelude::*;
use rand_xorshift::XorShiftRng;

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};

use pico_detect::{Clusterizer, Perturbator, utils::{Detection, Square}};

fn bench_clusterizer_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clusterizer::clusterize");
    let init = Square::new(100, 100, 100).into();

    for n in [10, 20, 30, 40, 50].iter() {
        let id = BenchmarkId::from_parameter(n);

        group.throughput(Throughput::Elements(*n as u64));

        group.bench_with_input(id, &n, |b, &n| {
            let mut perturbator = Perturbator::builder().build().unwrap();
            let mut clusterizer = Clusterizer::builder()
                .with_intersection_threshold(0.9)
                .build()
                .unwrap();
            let mut rng = XorShiftRng::seed_from_u64(42);

            perturbator.run(*n, init, |s| {
                let score = rng.sample(OpenClosed01);
                clusterizer.push(black_box(Detection::new(s.into(), score)));
            });

            b.iter(|| clusterizer.clusterize());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_clusterizer_run);
