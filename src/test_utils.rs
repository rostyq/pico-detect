use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use image::GrayImage;
use na::{Point2, Point3};

use super::localizer::Localizer;
use super::detector::Detector;
use super::shaper::Shaper;

pub fn create_test_image(width: u32, height: u32) -> GrayImage {
    use image::Luma;
    let mut image = GrayImage::new(width, height);
    image.put_pixel(0, 0, Luma::from([42u8]));
    image.put_pixel(width - 1, height - 1, Luma::from([255u8]));
    image
}

pub fn load_face_landmarks_model() -> Shaper {
    let fp = File::open("./models/shaper_5_face_landmarks.bin").unwrap();
    Shaper::from_readable(fp).unwrap()
}

pub fn load_puploc_model() -> Localizer {
    let fp = File::open("./models/puploc.bin").unwrap();
    Localizer::from_readable(fp).unwrap()
}

pub fn load_facefinder_model() -> Detector {
    let fp = File::open("./models/facefinder").unwrap();
    Detector::from_readable(fp).unwrap()
}

pub fn load_image(path: &Path) -> GrayImage {
    image::open(path).unwrap().to_luma()
}

pub fn load_test_image() -> (GrayImage, (Point3<u32>, Point2<f32>, Point2<f32>)){
    let assets_dir = Path::new("./assets/");
    let image_path = assets_dir.join("Lenna_(test_image).png");
    let image = load_image(&image_path);
    let data = load_data(&image_path.with_extension("txt"));
    (image, data)
}

pub fn create_init_point(point: &Point2<f32>) -> Point3<f32> {
    let mut init_point = point.xyx();
    init_point.x += 5.0;
    init_point.y += 5.0;
    init_point.z = 40.0;
    init_point
}

pub fn load_data(path: &Path) -> (Point3<u32>, Point2<f32>, Point2<f32>) {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut buf = String::new();
    reader.read_line(&mut buf).expect("no first line");
    buf.clear();

    reader.read_line(&mut buf).expect("no face data");
    let data = buf
        .trim()
        .split("\t")
        .filter_map(|s| s.parse::<u32>().ok())
        .collect::<Vec<_>>();
    let face = Point3::new(data[0], data[1], data[2]);

    reader.read_line(&mut buf).expect("no first line");
    buf.clear();

    reader.read_line(&mut buf).expect("no eyes data");
    let data = buf
        .trim()
        .split("\t")
        .filter_map(|s| s.parse::<f32>().ok())
        .collect::<Vec<_>>();

    let left_pupil = Point2::new(data[0], data[1]);
    let right_pupil = Point2::new(data[2], data[3]);

    (face, left_pupil, right_pupil)
}
