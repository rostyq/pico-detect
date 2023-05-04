mod common;

use rstest::rstest;
use approx::assert_abs_diff_eq;

use image::GrayImage;
use nalgebra::Point2;

use pico_detect::{Shaper, Square};

use common::{shaper, shaper_case};

#[rstest]
fn test_shaper_predict(shaper: Shaper, shaper_case: (GrayImage, Square, Vec<Point2<f32>>)) {
    let (image, region, points) = shaper_case;

    for (p1, p2) in shaper.shape(&image, region.into()).iter().zip(points.iter()) {
        assert_abs_diff_eq!(*p1, *p2, epsilon = 1e-4);
    }
}