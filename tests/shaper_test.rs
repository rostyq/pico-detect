#[macro_use]
mod common;

use image;

use pico_detect::{Shaper, utils::{Point2, Rect}};

#[macro_use]
extern crate approx;

#[test]
fn test_shaper_predict() {
    let shaper = load_model!(shaper);
    let image = load_test_image!();

    let test_points = vec![
        [341.8397, 269.6037],
        [318.1169, 272.2306],
        [253.2326, 266.5196],
        [284.6829, 271.6468],
        [306.5808, 331.5721],
    ];

    let size = 153;
    let left = 213;
    let top = 225;
    let rect = Rect::at(left, top).of_size(size, size);
    let points = dbg!(shaper.shape(&image, rect));

    for (point, test_data) in points.iter().zip(test_points.iter()) {
        assert_abs_diff_eq!(*point, Point2::from(*test_data), epsilon = 1e-4);
    }
}