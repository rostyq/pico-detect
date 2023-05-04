#[macro_use]
mod common;

use image;
use nalgebra::Point2;

use pico_detect::{utils::Square, Localizer, PerturbatingLocalizer};
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;

#[macro_use]
extern crate approx;

#[test]
fn test_perturbating_localizer_localize() {
    let image = load_test_image!();
    let localizer = PerturbatingLocalizer::builder()
        .with_perturbs(31)
        .build(load_model!(puploc))
        .unwrap();
    let mut rng = Xoroshiro128PlusPlus::seed_from_u64(42);

    assert_abs_diff_eq!(
        localizer.localize(&mut rng, &image, Square::new(300, 244, 38).into()),
        Point2::new(328.6757, 265.8514),
        epsilon = 1.0
    );

    assert_abs_diff_eq!(
        localizer.localize(&mut rng, &image, Square::new(250, 250, 39).into()),
        Point2::new(265.1674, 265.0339),
        epsilon = 1.0
    );
}