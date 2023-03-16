#[macro_export]
macro_rules! load_test_image {
    () => {
        image::open("./assets/test.png")
            .expect("failed to load test image")
            .to_luma8()
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! load_model {
    (facefinder) => {
        Detector::load(model_file!(facefinder))
            .expect("failed to load facefinder model")
    };

    (puploc) => {
        Localizer::load(model_file!(puploc))
            .expect("failed to load puploc model")
    };

    (shaper) => {
        Shaper::load(model_file!(shaper))
            .expect("failed to load shaper model")
    }
}