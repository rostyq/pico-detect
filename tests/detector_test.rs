mod common;

use approx::assert_abs_diff_eq;
use image::GrayImage;
use rstest::rstest;

use pico_detect::{DetectMultiscale, Detector, Square, Target};

use common::{classify_case, detect_multiscale, detect_multiscale_case, detector};

#[rstest]
fn test_detector_classify(detector: Detector, classify_case: (GrayImage, Square, Option<f32>)) {
    let (image, region, score) = classify_case;
    assert!(detector.classify(&image, region) == score);
}

#[rstest]
fn test_detect_multiscale(
    detect_multiscale: DetectMultiscale,
    detector: Detector,
    detect_multiscale_case: (GrayImage, Vec<(Target, f32)>),
) {
    let (image, detections) = detect_multiscale_case;

    for (detection, (target, score)) in detect_multiscale
        .run(&detector, &image)
        .iter()
        .zip(detections.iter())
    {
        assert_abs_diff_eq!(detection.score(), score, epsilon = 1e-4);
        assert_abs_diff_eq!(detection.region().size(), target.size(), epsilon = 1e-4);
        assert_abs_diff_eq!(detection.region().point(), target.point(), epsilon = 1e-4);
    }
}
