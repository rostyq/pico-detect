use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use rstest::fixture;

use image::GrayImage;
use nalgebra::Point2;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;

use pico_detect::{
    clusterize::Clusterizer, multiscale::Multiscaler, DetectMultiscale, Detector,
    LocalizePerturbate, Localizer, Padding, Shaper, Square, Target,
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

/// Environment variable with a path to the optional 68-point GTX shaper model.
pub const SHAPER_GTX_ENV: &str = "PICO_DETECT_SHAPER_GTX";

#[fixture]
pub fn shaper_gtx_path() -> Option<PathBuf> {
    std::env::var_os(SHAPER_GTX_ENV).map(PathBuf::from)
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

/// Optional 68-point GTX model converted from dlib's
/// `shape_predictor_68_face_landmarks_GTX.dat`. It is not part of the
/// repository; tests using it run only when [`SHAPER_GTX_ENV`] is set to
/// the model path and are skipped otherwise.
#[fixture]
pub fn shaper_gtx(shaper_gtx_path: Option<PathBuf>) -> Option<Shaper> {
    let Some(path) = shaper_gtx_path else {
        eprintln!("skipping: set {SHAPER_GTX_ENV} to the GTX shaper model path to run");
        return None;
    };
    let reader = BufReader::new(
        File::open(&path)
            .unwrap_or_else(|e| panic!("cannot open {SHAPER_GTX_ENV}={}: {e}", path.display())),
    );
    Some(Shaper::load(reader).expect("failed to load GTX shaper model"))
}

#[fixture]
pub fn classify_case(test_image: GrayImage) -> (GrayImage, Square, Option<f32>) {
    (
        test_image,
        Square::at(216, 208).of_size(170),
        Some(2.9913146),
    )
}

#[fixture]
pub fn localize_case(test_image: GrayImage) -> (GrayImage, [(Square, Point2<f32>); 2]) {
    (
        test_image,
        [
            (
                Square::at(321, 259).of_size(15),
                Point2::new(326.1010, 265.2927),
            ),
            (
                Square::at(259, 259).of_size(15),
                Point2::new(265.7270, 268.3979),
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
            [341.33582, 269.2085].into(),
            [317.94373, 271.2801].into(),
            [253.18811, 265.35452].into(),
            [284.09402, 270.37848].into(),
            [306.28616, 331.08255].into(),
        ],
    )
}

/// Landmarks predicted by dlib (`dlib.shape_predictor`) with the original
/// `shape_predictor_68_face_landmarks_GTX.dat` model on the same image and region.
#[fixture]
pub fn shaper_gtx_case(test_image: GrayImage) -> (GrayImage, Square, Vec<[i32; 2]>) {
    (
        test_image,
        Square::at(213, 225).of_size(153),
        vec![
            [204, 265],
            [202, 288],
            [203, 313],
            [207, 337],
            [217, 358],
            [234, 375],
            [251, 388],
            [271, 398],
            [290, 399],
            [304, 392],
            [313, 375],
            [321, 359],
            [330, 343],
            [338, 326],
            [345, 309],
            [348, 293],
            [347, 277],
            [239, 251],
            [253, 246],
            [269, 245],
            [284, 248],
            [298, 255],
            [324, 256],
            [331, 251],
            [339, 248],
            [346, 246],
            [351, 249],
            [310, 271],
            [312, 287],
            [314, 303],
            [316, 319],
            [292, 326],
            [300, 329],
            [308, 332],
            [316, 329],
            [322, 324],
            [253, 269],
            [263, 262],
            [275, 263],
            [283, 272],
            [272, 275],
            [261, 274],
            [317, 272],
            [325, 263],
            [335, 262],
            [340, 268],
            [335, 273],
            [326, 274],
            [268, 354],
            [284, 350],
            [298, 346],
            [305, 348],
            [310, 345],
            [315, 347],
            [317, 350],
            [312, 357],
            [307, 363],
            [301, 365],
            [294, 365],
            [281, 362],
            [274, 354],
            [297, 354],
            [303, 354],
            [309, 352],
            [314, 351],
            [308, 351],
            [303, 353],
            [296, 354],
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
        vec![(Target::new(292.4412, 300.9706, 161.7059), 39.9324)],
    )
}

pub fn file<P: AsRef<Path>>(path: P) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}
