use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use image::{DynamicImage, GrayImage};
use na::{Point2, Point3};

use super::localizer::Localizer;
use super::detector::Detector;

pub fn create_test_image(width: u32, height: u32) -> GrayImage {
    use image::Luma;
    let mut image = GrayImage::new(width, height);
    image.put_pixel(0, 0, Luma::from([42u8]));
    image.put_pixel(width - 1, height - 1, Luma::from([255u8]));
    image
}

pub fn load_puploc_model() -> Localizer {
    let fp = File::open("./models/puploc.bin").unwrap();
    Localizer::from_readable(fp).unwrap()
}

pub fn load_facefinder_model() -> Detector {
    let fp = File::open("./models/facefinder").unwrap();
    Detector::from_readable(fp).unwrap()
}

pub fn load_test_image(path: &Path) -> GrayImage {
    match image::open(path).unwrap() {
        DynamicImage::ImageLuma8(image) => image,
        _ => panic!("invalid test image"),
    }
}

pub fn create_init_point(point: &Point2<f32>) -> Point3<f32> {
    let mut init_point = point.xyx();
    init_point.x += 5.0;
    init_point.y += 5.0;
    init_point.z = 40.0;
    init_point
}

pub fn load_test_data(path: &Path) -> (Point2<f32>, Point2<f32>) {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut buf = String::new();
    reader.read_line(&mut buf).expect("no first line");
    buf.clear();

    reader.read_line(&mut buf).expect("no data");
    let data = buf
        .trim()
        .split("\t")
        .filter_map(|s| s.parse::<f32>().ok())
        .collect::<Vec<_>>();

    (Point2::new(data[0], data[1]), Point2::new(data[2], data[3]))
}
