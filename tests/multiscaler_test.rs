#[macro_use]
mod common;

use image;
use pico_detect::{
    utils::Region,
    Detector, MultiscaleDetector,
};

#[macro_use]
extern crate approx;

#[test]
fn test_multiscale_detector() {
    let image = load_test_image!();
    let mut detector = MultiscaleDetector::builder()
        .map_multiscale_builder(|b| {
            b.with_min_size(100)
                .with_max_size(image.width())
                .with_shift_factor(0.05)
                .with_scale_factor(1.1)
        })
        .map_clusterizer_builder(|b| {
            b.with_intersection_threshold(0.2)
                .with_score_threshold(30.0)
        })
        .build(load_model!(facefinder))
        .expect("failed to build multiscale detector");

    let detections = dbg!(detector.detect(&image));
    assert_eq!(detections.len(), 1);

    let detection = detections[0];

    assert_abs_diff_eq!(detection.region().left(), 290 - 154 / 2 - 1);
    assert_abs_diff_eq!(detection.region().top(), 302 - 154 / 2 - 1);

    assert_eq!(detection.region().width(), 154);
    assert_eq!(detection.region().height(), 154);

    assert_abs_diff_eq!(detection.score(), 38.2221, epsilon = 1e-4);
}
