extern crate image;
extern crate nalgebra;
extern crate pico_detect;

use image::{Rgb, RgbImage};
use nalgebra::{Point2, Point3};
use pico_detect::{create_xorshift_rng, CascadeParameters, Detector, Localizer};
use std::{env, include_bytes};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("should have `in` and `out` path");
    }

    let (input, output) = (&args[1], &args[2]);

    let facefinder_bin = include_bytes!("../models/facefinder").to_vec();
    let puploc_bin = include_bytes!("../models/puploc.bin").to_vec();

    // load detectors
    let facefinder = Detector::from_readable(facefinder_bin.as_slice()).unwrap();
    let puploc = Localizer::from_readable(puploc_bin.as_slice()).unwrap();

    // source of "randomness" for perturbated search for pupil
    let mut rng = create_xorshift_rng(42u64);
    let nperturbs = 31usize;

    // load image
    let dyn_image = image::open(input).unwrap();
    let (gray, mut image) = (dyn_image.to_luma(), dyn_image.to_rgb());

    // parameters for face detection
    let params = CascadeParameters::new(
        (gray.width() / 10) as usize,
        (gray.width() / 2) as usize,
        0.05,
        1.1,
    );

    let detections = facefinder.find_clusters(&gray, &params, 0.05);

    println!("Faces detected: {}.", detections.len());
    for (i, detection) in detections.iter().enumerate() {
        println!(
            "{} :: point: {}; score: {}",
            i, &detection.point, detection.score
        );
        let center = &detection.point;
        let size = detection.point.z;
        let right_pupil = Point3::new(center.x - size * 0.1, center.y - size * 0.2, size * 0.33);
        let left_pupil = Point3::new(center.x + size * 0.1, center.y - size * 0.2, size * 0.33);

        // find pupils
        let right_pupil = puploc.perturb_localize(&gray, &right_pupil, &mut rng, nperturbs);
        let left_pupil = puploc.perturb_localize(&gray, &left_pupil, &mut rng, nperturbs);

        // draw red cross on the image
        draw_cross(&mut image, &right_pupil);
        draw_cross(&mut image, &left_pupil);
    }

    image.save(output).unwrap();
}

fn draw_cross(image: &mut RgbImage, center: &Point2<f32>) {
    let x = center.x.round() as i32;
    let y = center.y.round() as i32;
    for i in -2..3 {
        for j in -2..3 {
            if i == 0 || j == 0 {
                image.put_pixel((x + i) as u32, (y + j) as u32, Rgb([255, 0, 0]));
            }
        }
    }
}
