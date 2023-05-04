#[path = "./common/macros.rs"]
mod macros;

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};

use pico_detect::{utils::Square, Perturbator};
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;

fn bench_perturbator_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Perturbator::run");

    let perturbator = Perturbator::builder().build().unwrap();
    let mut rng = Xoroshiro128PlusPlus::seed_from_u64(42);

    let init = Square::new(100, 100, 100).into();

    for n in [15, 19, 23, 27, 31].iter() {
        let id = BenchmarkId::from_parameter(n);

        group.throughput(Throughput::Elements(*n as u64));

        group.bench_with_input(id, &n, |b, &n| {
            b.iter(|| {
                perturbator.run(&mut rng, *n, init, |s| {
                    black_box(s);
                })
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_perturbator_run);
