extern crate pico_detect;

use pico_detect::CascadeParameters;
use pico_detect::test_utils::{load_facefinder_model, load_test_image};

fn main() {
    let facefinder = load_facefinder_model();
    let (image, _data) = load_test_image();

    let params = CascadeParameters::new(150, 300, 0.05, 1.05);
    let detections = facefinder.find_clusters(&image, &params, 0.1);

    for (i, detection) in detections.iter().enumerate() {
        println!(
            "{} :: point: {}; score: {}",
            i, &detection.point, detection.score
        );
    }
}
