extern crate image;
extern crate imageproc;
extern crate nalgebra;
extern crate pico_detect;

use image::{GrayImage, Rgb, RgbImage};
use imageproc::drawing;
use imageproc::rect::Rect;
use nalgebra::{Point2, Point3};
use pico_detect::{create_xorshift_rng, CascadeParameters, Detector, Localizer, Shaper};
use std::include_bytes;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "pico-detect-cli")]
struct Opt {
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
    #[structopt(long, default_value = "100")]
    min_size: u32,
    #[structopt(long, default_value = "1.1")]
    scale_factor: f32,
    #[structopt(long, default_value = "0.05")]
    shift_factor: f32,
}

struct Face {
    score: f32,
    roi: Point3<f32>,
    shape: Vec<Point2<f32>>,
    pupils: (Point2<f32>, Point2<f32>),
}

fn main() {
    let opt = Opt::from_args();
    let dyn_image = image::open(&opt.input).expect("Cannot open input image.");
    let (gray, mut image) = (dyn_image.to_luma(), dyn_image.to_rgb());

    let (facefinder, shaper, puploc) = load_models();

    let faces = detect_faces(&opt, &gray, &facefinder, &shaper, &puploc);

    if opt.verbose {
        print_faces_data(&faces);
    }

    for face in faces.iter() {
        draw_face(&mut image, &face);
    }

    image.save(opt.output).expect("Cannot write output.");
}

fn load_models() -> (Detector, Shaper, Localizer) {
    let facefinder_bin = include_bytes!("../models/facefinder").to_vec();
    let puploc_bin = include_bytes!("../models/puploc.bin").to_vec();
    let shaper_bin = include_bytes!("../models/shaper_5_face_landmarks.bin").to_vec();

    let facefinder = Detector::from_readable(facefinder_bin.as_slice()).unwrap();
    let puploc = Localizer::from_readable(puploc_bin.as_slice()).unwrap();
    let shaper = Shaper::from_readable(shaper_bin.as_slice()).unwrap();

    (facefinder, shaper, puploc)
}

fn detect_faces(
    opt: &Opt,
    gray: &GrayImage,
    detector: &Detector,
    shaper: &Shaper,
    localizer: &Localizer,
) -> Vec<Face> {
    // parameters for face detection
    let cascade_params = CascadeParameters::new(
        opt.min_size,
        gray.width(),
        opt.shift_factor,
        opt.scale_factor,
    );

    // source of "randomness" for perturbated search for pupil
    let mut rng = create_xorshift_rng(42u64);
    let nperturbs = 31usize;

    let detections = detector.find_clusters(&gray, &cascade_params, 0.2);

    detections
        .iter()
        .filter_map(|detection| {
            if detection.score < 40.0 {
                return None;
            }
            let shape = shaper.predict(&gray, &detection.point);
            let pupils = Shape5::find_eyes_roi(&shape);
            let pupils = (
                localizer.perturb_localize(&gray, &pupils.0, &mut rng, nperturbs),
                localizer.perturb_localize(&gray, &pupils.1, &mut rng, nperturbs),
            );

            Some(Face {
                score: detection.score,
                roi: detection.point,
                shape,
                pupils,
            })
        })
        .collect::<Vec<Face>>()
}

fn draw_face(image: &mut RgbImage, face: &Face) {
    let hs = face.roi.z / 2.0;
    let rect = Rect::at(
        (face.roi.x - hs) as i32,
        (face.roi.y - hs) as i32,
    )
    .of_size(face.roi.z as u32, face.roi.z as u32);

    drawing::draw_hollow_rect_mut(image, rect, Rgb([0, 0, 255]));
    for (_i, point) in face.shape.iter().enumerate() {
        drawing::draw_cross_mut(
            image,
            Rgb([0, 255, 0]),
            point.x as i32,
            point.y as i32,
        );
    }

    drawing::draw_cross_mut(
        image,
        Rgb([255, 0, 0]),
        face.pupils.0.x as i32,
        face.pupils.0.y as i32,
    );
    drawing::draw_cross_mut(
        image,
        Rgb([255, 0, 0]),
        face.pupils.1.x as i32,
        face.pupils.1.y as i32,
    );
}

fn print_faces_data(faces: &[Face]) {
    println!("Faces detected: {}.", faces.len());
    for (i, face) in faces.iter().enumerate() {
        println!(
            "{} :: point: {}; score: {}",
            i, &face.roi, face.score
        );

        for (i, point) in face.shape.iter().enumerate() {
            println!("\tlandmark {}: {}", i, &point);
        }

        println!("\tleft  pupil: {}", &face.pupils.0);
        println!("\tright pupil: {}", &face.pupils.1);
    }
}

enum Shape5 {
    LeftOuterEyeCorner = 0,
    LeftInnerEyeCorner = 1,
    RightOuterEyeCorner = 2,
    RightInnerEyeCorner = 3,
    #[allow(dead_code)]
    Nose = 4,
}

impl Shape5 {
    fn size() -> usize {
        5
    }

    #[allow(dead_code)]
    fn find_eye_centers(shape: &[Point2<f32>]) -> (Point2<f32>, Point2<f32>) {
        assert_eq!(shape.len(), Self::size());
        (
            nalgebra::center(
                &shape[Self::LeftInnerEyeCorner as usize],
                &shape[Self::LeftOuterEyeCorner as usize],
            ),
            nalgebra::center(
                &shape[Self::RightInnerEyeCorner as usize],
                &shape[Self::RightOuterEyeCorner as usize],
            ),
        )
    }

    fn find_eyes_roi(shape: &[Point2<f32>]) -> (Point3<f32>, Point3<f32>) {
        assert_eq!(shape.len(), Self::size());
        let (li, lo) = (
            &shape[Self::LeftInnerEyeCorner as usize],
            &shape[Self::LeftOuterEyeCorner as usize],
        );
        let (ri, ro) = (
            &shape[Self::RightInnerEyeCorner as usize],
            &shape[Self::RightOuterEyeCorner as usize],
        );

        let (dl, dr) = (lo - li, ri - ro);
        let (l, r) = (li + dl.scale(0.5), ro + dr.scale(0.5));
        let mut l = l.to_homogeneous();
        l.z = dl.norm() * 1.1;
        let mut r = r.to_homogeneous();
        r.z = dr.norm() * 1.1;

        (l.into(), r.into())
    }
}
