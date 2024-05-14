use std::{
    fs::File,
    path::{Path, PathBuf}, io::BufReader,
};

use rstest::fixture;

use image::GrayImage;
use nalgebra::Point2;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;

use pico_detect::{
    clusterize::Clusterizer, multiscale::Multiscaler, DetectMultiscale, Detector,
    Localizer, Padding, Shaper, Square, Target, LocalizePerturbate,
};

#[fixture]
pub fn test_image_path() -> PathBuf {
    "./assets/test.png".into()
}

#[fixture]
pub fn detector_path() -> PathBuf {
    "./models/face.detector.bin".into()
}

#[fixture]
pub fn localizer_path() -> PathBuf {
    "./models/pupil.localizer.bin".into()
}

#[fixture]
pub fn shaper_path() -> PathBuf {
    "./models/face-5.shaper.bin".into()
}

#[fixture]
pub fn test_image(test_image_path: PathBuf) -> image::GrayImage {
    image::open(test_image_path).unwrap().to_luma8()
}

#[fixture]
pub fn detector(detector_path: PathBuf) -> Detector {
    Detector::load(file(detector_path)).unwrap()
}

#[fixture]
pub fn localizer(localizer_path: PathBuf) -> Localizer {
    Localizer::load(file(localizer_path)).unwrap()
}

#[fixture]
pub fn shaper(shaper_path: PathBuf) -> Shaper {
    Shaper::load(file(shaper_path)).unwrap()
}

#[fixture]
pub fn classify_case(test_image: GrayImage) -> (GrayImage, Square, Option<f32>) {
    (
        test_image,
        Square::at(216, 208).of_size(170),
        Some(2.4434934),
    )
}

#[fixture]
pub fn localize_case(test_image: GrayImage) -> (GrayImage, [(Square, Point2<f32>); 2]) {
    (
        test_image,
        [
            (
                Square::at(321, 259).of_size(15),
                Point2::new(326.8915, 266.5068),
            ),
            (
                Square::at(259, 259).of_size(15),
                Point2::new(266.5190, 267.5272),
            ),
        ],
    )
}

#[fixture]
pub fn shaper_case(test_image: GrayImage) -> (GrayImage, Square, Vec<Point2<f32>>) {
    (
        test_image,
        Square::at(213, 225).of_size(153),
        vec![
            [341.8397, 269.6037].into(),
            [318.1169, 272.2306].into(),
            [253.2326, 266.5196].into(),
            [284.6829, 271.6468].into(),
            [306.5808, 331.5721].into(),
        ],
    )
}

#[fixture]
pub fn multiscaler(test_image: GrayImage) -> Multiscaler {
    Multiscaler::new(100, test_image.width(), 0.05, 1.1).unwrap()
}

#[fixture]
pub fn clusterizer() -> Clusterizer {
    Clusterizer {
        intersection_threshold: 0.2,
        score_threshold: 30.0,
    }
}

#[fixture]
pub fn padding() -> Padding {
    Default::default()
}

#[fixture]
pub fn detect_multiscale(
    multiscaler: Multiscaler,
    clusterizer: Clusterizer,
    padding: Padding,
) -> DetectMultiscale {
    DetectMultiscale::builder()
        .multiscaler(multiscaler)
        .clusterizer(clusterizer)
        .padding(padding)
        .build()
        .unwrap()
}

#[fixture]
pub fn localize_perturbate() -> LocalizePerturbate {
    LocalizePerturbate::new(31)
}

#[fixture]
pub fn rng() -> Xoroshiro128PlusPlus {
    Xoroshiro128PlusPlus::seed_from_u64(42)
}

#[fixture]
pub fn localize_perturbate_case(test_image: GrayImage) -> (GrayImage, [(Square, Point2<f32>); 2]) {
    (
        test_image,
        [
            (
                Square::at(300, 244).of_size(38),
                Point2::new(328.6757, 265.8514),
            ),
            (
                Square::at(250, 250).of_size(38),
                Point2::new(265.1674, 265.0339),
            ),
        ],
    )
}

#[fixture]
pub fn detect_multiscale_case(test_image: GrayImage) -> (GrayImage, Vec<(Target, f32)>) {
    (
        test_image,
        vec![(Target::new(289.7143, 302.0000, 154.5714), 38.2221)],
    )
}

pub fn file<P: AsRef<Path>>(path: P) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}
