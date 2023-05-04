#[path = "./common/macros.rs"]
mod macros;

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};

use rand::prelude::*;

use pico_detect::{clusterize::Clusterizer, perturbate::Perturbator, Detection, Square};
use rand_xoshiro::Xoroshiro128PlusPlus;

fn bench_clusterizer_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Clusterizer::clusterize");
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

criterion_group!(benches, bench_clusterizer_run);
