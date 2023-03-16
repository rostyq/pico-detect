#[macro_use]
mod common;

use std::fmt::Display;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use pico_detect::utils::Multiscaler;

#[derive(Clone, Copy, Debug)]
struct Size {
    width: u32,
    height: u32,
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl From<(u32, u32)> for Size {
    fn from(value: (u32, u32)) -> Self {
        Self { width: value.0, height: value.1 }
    }
}

fn bench_multiscale_run(c: &mut Criterion) {
    static SIZES: &[(u32, u32)] = &[
        (320, 240),
        (480, 360),
        (640, 480),
        (1280, 720),
        (1920, 1280),
    ];

    let mut group = c.benchmark_group("Multiscaler::run");

    for size in SIZES.iter().map(|s| Size::from(*s)) {
        let ms = Multiscaler::builder()
            .with_min_size(100)
            .with_max_size(size.height)
            .build()
            .unwrap();

        let id = BenchmarkId::from_parameter(size);

        group.throughput(Throughput::Elements(ms.count(size.width, size.height) as u64));

        group.bench_with_input(id, &size, |b, &s| {
            b.iter(||  ms.run(s.width, s.height, |s| {
                black_box(s);
            }))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_multiscale_run);
criterion_main!(benches);
