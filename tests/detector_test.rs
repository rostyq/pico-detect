#[macro_use]
mod common;

use image;
use pico_detect::{utils::Square, Detector};

#[macro_use]
extern crate approx;

#[test]
fn test_detector_detect() {
    let detector = load_model!(facefinder);

    let image = load_test_image!();

    let score = detector.classify(&image, Square::at(216, 208).of_size(170));
    assert!(score.is_some());
    assert_abs_diff_eq!(score.unwrap(), 2.4434934);
}
