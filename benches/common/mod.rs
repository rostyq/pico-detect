#[macro_export]
macro_rules! load_test_image {
    () => {
        image::open("./assets/test.png")
            .expect("failed to load test image")
            .to_luma8()
    };
}

#[macro_export]
macro_rules! model_path {
    (facefinder) => {
        std::path::Path::new("./models/facefinder")
    };

    (puploc) => {
        std::path::Path::new("./models/puploc.bin")
    };

    (shaper) => {
        std::path::Path::new("./models/shaper_5_face_landmarks.bin")
    };
}

#[macro_export]
macro_rules! model_file {
    (facefinder) => {
        std::fs::File::open("./models/facefinder")
            .expect("cannot open facefinder model file")
    };

    (puploc) => {
        std::fs::File::open("./models/puploc.bin")
            .expect("cannot open puploc model file")
    };

    (shaper) => {
        std::fs::File::open("./models/shaper_5_face_landmarks.bin")
            .expect("cannot open shaper model file")
    };
}

#[macro_export]
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