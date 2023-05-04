mod common;

use approx::assert_abs_diff_eq;
use image::GrayImage;
use rstest::rstest;

use nalgebra::Point2;

use common::{localize_case, localize_perturbate_case, localizer, localize_perturbate, rng};

use pico_detect::{Localizer, Square, LocalizePerturbate};

#[rstest]
fn test_localizer_localize(
    localizer: Localizer,
    localize_case: (GrayImage, [(Square, Point2<f32>); 2]),
) {
    let (image, tests) = localize_case;

    for (region, point) in tests.iter() {
        assert_abs_diff_eq!(
            localizer.localize(&image, region.to_owned().into()),
            point,
            epsilon = 1e-4
        );
    }
}

#[rstest]
fn test_localize_perturbate_run(
    localizer: Localizer,
    mut rng: rand_xoshiro::Xoroshiro128PlusPlus,
    localize_perturbate: LocalizePerturbate,
    localize_perturbate_case: (GrayImage, [(Square, Point2<f32>); 2]),
) {
    let (image, tests) = localize_perturbate_case;

    for (region, point) in tests.iter() {
        assert_abs_diff_eq!(
            localize_perturbate.run(&localizer, &mut rng, &image, region.to_owned().into()),
            point,
            epsilon = 1.0
        );
    }
}
