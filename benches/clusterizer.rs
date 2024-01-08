#[path = "./common/macros.rs"]
mod macros;

use std::time::Duration;

use criterion::{black_box, BenchmarkId, Criterion, Throughput};

use rand::prelude::*;

use pico_detect::{clusterize::Clusterizer, perturbate::Perturbator, Detection, Square};
use rand_xoshiro::Xoroshiro128PlusPlus;

pub fn bench_clusterize(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clusterizer::clusterize");

    group.warm_up_time(Duration::from_secs(5));
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(15));

    let init = Square::new(100, 100, 100).into();

    for n in [10, 30, 50].iter() {
        let id = BenchmarkId::from_parameter(n);

        group.throughput(Throughput::Elements(*n as u64));

        group.bench_with_input(id, &n, |b, &n| {
            let perturbator = Perturbator::default();
            let clusterizer = Clusterizer::default();

            let mut rng = Xoroshiro128PlusPlus::seed_from_u64(42);

            let mut data = Vec::with_capacity(*n);

            perturbator.run(&mut rng, *n, init, |s| {
                data.push(Detection::new(s.into(), 1.0));
            });

            b.iter(|| {
                clusterizer.clusterize(
                    black_box(data.to_owned()).as_mut_slice(),
                    &mut Vec::with_capacity(*n),
                )
            });
        });
    }

    group.finish();
}
