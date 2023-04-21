#[macro_use]
extern crate criterion;

mod clusterizer_bench;
mod multiscaler_bench;
mod perturbator_bench;
mod detector_bench;
mod localizer_bench;
mod shaper_bench;

criterion_main!(
    clusterizer_bench::benches,
    multiscaler_bench::benches,
    perturbator_bench::benches,
    detector_bench::benches,
    localizer_bench::benches,
    shaper_bench::benches,
);