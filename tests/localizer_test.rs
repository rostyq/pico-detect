#[macro_use]
mod common;

use image;
use nalgebra::Point2;

use pico_detect::{utils::Square, Localizer, PerturbatingLocalizer};

#[macro_use]
extern crate approx;

#[test]
fn test_localizer_localize() {
    let image = load_test_image!();
    let localizer = load_model!(puploc);

    assert_abs_diff_eq!(
        localizer.localize(&image, Square::new(321, 259, 15).into()),
        Point2::new(326.8915, 266.5068),
        epsilon = 1e-4
    );

    assert_abs_diff_eq!(
        localizer.localize(&image, Square::new(259, 259, 15).into()),
        Point2::new(266.5190, 267.5272),
        epsilon = 1e-4
    );
}

#[test]
fn test_perturbating_localizer_localize() {
    let image = load_test_image!();
    let mut localizer = PerturbatingLocalizer::builder()
        .with_perturbs(31)
        .build(load_model!(puploc))
        .unwrap();

    assert_abs_diff_eq!(
        localizer.localize(&image, Square::new(300, 244, 38).into()),
        Point2::new(328.6757, 265.8514),
        epsilon = 1e-4
    );

    assert_abs_diff_eq!(
        // localizer.localize(&image, Square::new(255, 255, 35).into()),
        localizer.localize(&image, Square::new(250, 250, 39).into()),
        Point2::new(265.1674, 265.0339),
        epsilon = 1e-4
    );
}
