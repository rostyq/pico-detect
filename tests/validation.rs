use image::GrayImage;
use nalgebra::{Point2, Point3};

#[cfg(test)]
#[macro_use]
extern crate approx;

const FACE_TEST: [u32; 3] = [302, 294, 170];
const FACE_ROI: [f32; 3] = [290.0, 302.0, 154.0];

const L_PUPIL_ROI: [f32; 3] = [328. + 5., 265. + 5., 40.];
const R_PUPIL_ROI: [f32; 3] = [265. + 5., 265. + 5., 40.];

const L_PUPIL: [f32; 2] = [328., 265.];
const R_PUPIL: [f32; 2] = [265., 265.];

const SHAPE: [[f32; 2]; 5] = [
    [341.8676, 269.91626],
    [318.3374, 271.74197],
    [253.79659, 265.64597],
    [285.11893, 270.44794],
    [306.6246, 331.8988],
];

#[test]
fn validate_facefinder() {
    use pico_detect::{CascadeParameters, Detection, Detector};

    let facefinder =
        Detector::from_readable(include_bytes!("../models/facefinder").to_vec().as_slice())
            .unwrap();
    let image = load_test_image();
    let score = facefinder.classify_region(&image, &Point3::from(FACE_TEST));
    assert!(score.is_some());
    assert_abs_diff_eq!(score.unwrap(), 2.4434934);

    let params = CascadeParameters::new(100, image.width(), 0.05, 1.1);
    let detections = facefinder.find_clusters(&image, &params, 0.2);
    let detections: Vec<Detection> = detections.into_iter().filter(|d| d.score > 40.0).collect();

    assert_eq!(detections.len(), 1);
    let detection = &detections[0];

    assert_abs_diff_eq!(detection.point, &Point3::from(FACE_ROI), epsilon = 1.0);
    assert_abs_diff_eq!(detection.score, 58.0, epsilon = 1.0);
}

#[test]
fn validate_face_landmarks() {
    use pico_detect::Shaper;

    let shaper = Shaper::from_readable(
        include_bytes!("../models/shaper_5_face_landmarks.bin")
            .to_vec()
            .as_slice(),
    )
    .unwrap();
    let tests: Vec<Point2<f32>> = SHAPE.iter().map(|d| Point2::new(d[0], d[1])).collect();
    let image = load_test_image();

    let shape = shaper.predict(&image, &Point3::from(FACE_ROI));
    assert_eq!(tests.len(), shape.len());

    for (predicted, test) in shape.iter().zip(tests.iter()) {
        assert_abs_diff_eq!(predicted, test, epsilon = 0.5);
    }
}

#[test]
fn validate_pupil_localization() {
    use pico_detect::{create_xorshift_rng, Localizer};
    let puploc =
        Localizer::from_readable(include_bytes!("../models/puploc.bin").to_vec().as_slice())
            .unwrap();
    let image = load_test_image();

    let epsilon = 3f32;
    assert_abs_diff_eq!(
        Point2::from(L_PUPIL),
        puploc.localize(&image, &Point3::from(L_PUPIL_ROI)),
        epsilon = epsilon
    );

    assert_abs_diff_eq!(
        Point2::from(R_PUPIL),
        puploc.localize(&image, &Point3::from(R_PUPIL_ROI)),
        epsilon = epsilon
    );

    let mut rng = create_xorshift_rng(42u64);

    let epsilon = 1.5f32;
    let nperturbs = 31usize;
    assert_abs_diff_eq!(
        Point2::from(L_PUPIL),
        puploc.perturb_localize(&image, &Point3::from(L_PUPIL_ROI), &mut rng, nperturbs),
        epsilon = epsilon
    );

    assert_abs_diff_eq!(
        Point2::from(R_PUPIL),
        puploc.perturb_localize(&image, &Point3::from(R_PUPIL_ROI), &mut rng, nperturbs),
        epsilon = epsilon
    );
}

pub fn load_test_image() -> GrayImage {
    image::open("./tests/assets/Lenna_(test_image).png")
        .unwrap()
        .to_luma()
}
