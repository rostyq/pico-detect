#![macro_use]
#![allow(unused_macros)]

macro_rules! load_test_image {
    () => {
        image::open("./assets/test.png")
            .expect("failed to load test image")
            .to_luma8()
    };
}

macro_rules! model_path {
    (facefinder) => {
        std::path::Path::new("./models/face.detector.bin")
    };

    (puploc) => {
        std::path::Path::new("./models/pupil.localizer.bin")
    };

    (shaper) => {
        std::path::Path::new("./models/face-5.shaper.bin")
    };
}

macro_rules! model_file {
    (facefinder) => {
        std::fs::File::open("./models/face.detector.bin")
            .expect("cannot open facefinder model file")
    };

    (puploc) => {
        std::fs::File::open("./models/pupil.localizer.bin")
            .expect("cannot open puploc model file")
    };

    (shaper) => {
        std::fs::File::open("./models/face-5.shaper.bin")
            .expect("cannot open shaper model file")
    };
}

macro_rules! load_model {
    (facefinder) => {
        pico_detect::Detector::load(model_file!(facefinder))
            .expect("failed to load facefinder model")
    };

    (puploc) => {
        pico_detect::Localizer::load(model_file!(puploc))
            .expect("failed to load puploc model")
    };

    (shaper) => {
        pico_detect::Shaper::load(model_file!(shaper))
            .expect("failed to load shaper model")
    }
}