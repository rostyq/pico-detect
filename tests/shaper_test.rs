mod common;

use approx::assert_abs_diff_eq;
use rstest::rstest;

use image::GrayImage;
use nalgebra::Point2;

use pico_detect::{Shaper, Square};

use common::{shaper, shaper_case, shaper_gtx, shaper_gtx_case};

#[rstest]
fn test_shaper_predict(shaper: Shaper, shaper_case: (GrayImage, Square, Vec<Point2<f32>>)) {
    let (image, region, points) = shaper_case;

    for (p1, p2) in shaper
        .shape(&image, region.into())
        .iter()
        .zip(points.iter())
    {
        assert_abs_diff_eq!(*p1, *p2, epsilon = 1e-4);
    }
}

#[rstest]
fn test_shaper_gtx_load(shaper_gtx: Option<Shaper>) {
    let Some(shaper_gtx) = shaper_gtx else { return };
    assert_eq!(shaper_gtx.size(), 68);
}

#[rstest]
fn test_shaper_gtx_predict_matches_dlib(
    shaper_gtx: Option<Shaper>,
    shaper_gtx_case: (GrayImage, Square, Vec<[i32; 2]>),
) {
    let Some(shaper_gtx) = shaper_gtx else { return };
    let (image, region, points) = shaper_gtx_case;

    let shape = shaper_gtx.shape(&image, region.into());
    assert_eq!(shape.len(), points.len());

    // dlib rounds predicted landmarks to the nearest pixel.
    for (i, (p, [x, y])) in shape.iter().zip(points.iter()).enumerate() {
        assert_eq!(
            [p.x.round() as i32, p.y.round() as i32],
            [*x, *y],
            "landmark {i} differs from dlib: {p}"
        );
    }
}
