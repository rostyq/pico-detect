use image::GrayImage;
use nalgebra::Point2;

#[cfg(test)]
#[macro_use]
extern crate approx;

const FACE_CENTER: [f32; 2] = [290.0, 302.0];
const FACE_SIZE: f32 = 154.0;

const L_PUPIL: [f32; 2] = [328., 265.];
const R_PUPIL: [f32; 2] = [265., 265.];
const SCALE: f32 = 40.0;

const SHAPE: [[f32; 2]; 5] = [
    [341., 269.],
    [318., 271.],
    [253., 265.],
    [285., 270.],
    [306., 331.],
];

#[test]
fn validate_facefinder() {
    use pico_detect::{Detection, Detector, ISimilarity2, MultiScale};

    let facefinder =
        Detector::from_readable(include_bytes!("../models/facefinder").to_vec().as_slice())
            .unwrap();
    let image = load_test_image();

    let score = facefinder.classify(&image, ISimilarity2::from_components(302, 294, 170));
    assert!(score.is_some());
    assert_abs_diff_eq!(score.unwrap(), 2.4434934);

    let multiscale = MultiScale::default()
        .with_size_range(100, image.width())
        .with_shift_factor(0.05)
        .with_scale_factor(1.1);
    let mut detections = multiscale.run(&facefinder, &image);
    let detections: Vec<Detection> = Detection::clusterize(&mut detections, 0.2)
        .into_iter()
        .filter(|d| d.score() > 40.0)
        .collect();

    assert_eq!(detections.len(), 1);
    let detection = &detections[0];

    assert_abs_diff_eq!(detection.center(), Point2::from(FACE_CENTER), epsilon = 1.0);
    assert_abs_diff_eq!(detection.size(), FACE_SIZE, epsilon = 1.0);
    assert_abs_diff_eq!(detection.score(), 58.0, epsilon = 1.0);
}

#[test]
fn validate_face_landmarks() {
    use pico_detect::{Rect, Shaper};

    let mut shaper = Shaper::from_readable(
        include_bytes!("../models/shaper_5_face_landmarks.bin")
            .to_vec()
            .as_slice(),
    )
    .unwrap();
    let tests: Vec<Point2<f32>> = SHAPE.iter().map(|d| Point2::new(d[0], d[1])).collect();
    let image = load_test_image();

    let size = FACE_SIZE as u32 - 1;
    let x = FACE_CENTER[0] - FACE_SIZE / 2.;
    let y = FACE_CENTER[1] - FACE_SIZE / 2.;
    let rect = Rect::at(x as i32, y as i32).of_size(size, size);
    let shape = shaper.predict(&image, rect);

    assert_eq!(tests.len(), shape.len());

    for (predicted, test) in shape.iter().zip(tests.iter()) {
        assert_abs_diff_eq!(predicted, test, epsilon = 1.0);
    }
}

#[test]
fn validate_pupil_localization() {
    use nalgebra::{Isometry2, Similarity2};
    use pico_detect::Localizer;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    let mut rng = XorShiftRng::seed_from_u64(42u64);

    let puploc =
        Localizer::from_readable(include_bytes!("../models/puploc.bin").to_vec().as_slice())
            .unwrap();

    let image = load_test_image();
    let left_pupil = Point2::from(L_PUPIL);
    let right_pupil = Point2::from(R_PUPIL);

    let left_roi = Similarity2::from_isometry(
        Isometry2::translation(left_pupil.x + 5., left_pupil.y + 5.),
        SCALE,
    );

    let right_roi = Similarity2::from_isometry(
        Isometry2::translation(right_pupil.x + 5., right_pupil.y + 5.),
        SCALE,
    );

    let epsilon = 3f32;
    assert_abs_diff_eq!(
        left_pupil,
        puploc.localize(&image, left_roi),
        epsilon = epsilon
    );

    assert_abs_diff_eq!(
        right_pupil,
        puploc.localize(&image, right_roi),
        epsilon = epsilon
    );

    let epsilon = 1.5f32;
    let nperturbs = 31usize;

    assert_abs_diff_eq!(
        left_pupil,
        puploc.perturb_localize(&image, left_roi, &mut rng, nperturbs),
        epsilon = epsilon
    );

    assert_abs_diff_eq!(
        right_pupil,
        puploc.perturb_localize(&image, right_roi, &mut rng, nperturbs),
        epsilon = epsilon
    );
}

pub fn load_test_image() -> GrayImage {
    image::open("./tests/assets/Lenna_(test_image).png")
        .unwrap()
        .to_luma()
}
