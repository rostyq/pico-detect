extern crate pico_detect;

mod args;
#[macro_use]
mod models;
mod init;
mod shape;
mod face;
mod utils;

use anyhow::{Context, Result};

use shape::Shape5;
use face::Face;
use utils::{print_faces_data, draw_face};

fn main() -> Result<()> {
    let args = args::parse();

    let image = image::open(&args.input).context("Failed to load image file.")?;

    let (mut detector, shaper, mut localizer) = init::initialize(&args, &image)?;

    let gray = image.to_owned().into_luma8();

    let faces: Vec<Face> = detector.detect(&gray).iter().map(|d| {
        let roi = *d.region();

        let shape = shaper.shape(&gray, roi.into());

        let (left_eye_roi, right_eye_roi) = Shape5::find_eyes_roi(&shape);
        let left_pupil = localizer.localize(&gray, left_eye_roi);
        let right_pupil = localizer.localize(&gray, right_eye_roi);

        Face {
            region: roi,
            shape,
            score: d.score(),
            pupils: (left_pupil, right_pupil),
        }
    })
    .collect();

    if args.verbose {
        print_faces_data(&faces);
    }

    let mut rgb = image.into_rgb8();
 
    for face in faces.iter() {
        draw_face(&mut rgb, &face);
    }

    rgb.save(args.output).context("Cannot write output image")?;

    Ok(())
}
