#[path = "./common/macros.rs"]
mod macros;

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};

use rand::prelude::*;

use pico_detect::{Clusterizer, Perturbator, utils::{Detection, Square}};
use rand_xoshiro::Xoroshiro128PlusPlus;

fn bench_clusterizer_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clusterizer::clusterize");
    let init = Square::new(100, 100, 100).into();

    for n in [10, 30, 50].iter() {
        let id = BenchmarkId::from_parameter(n);

        group.throughput(Throughput::Elements(*n as u64));

        group.bench_with_input(id, &n, |b, &n| {
            let perturbator = Perturbator::builder().build().unwrap();
            let mut clusterizer = Clusterizer::builder()
                .with_intersection_threshold(0.9)
                .build()
                .unwrap();
            let mut rng = Xoroshiro128PlusPlus::seed_from_u64(42);

            perturbator.run(&mut rng, *n, init, |s| {
                clusterizer.push(black_box(Detection::new(s, 1.0)));
            });

            b.iter(|| clusterizer.clusterize());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_clusterizer_run);
