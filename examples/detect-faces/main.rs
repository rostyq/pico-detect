extern crate pico_detect;

mod args;
#[macro_use]
mod models;
mod face;
mod init;
mod shape;
mod utils;

use anyhow::{anyhow, Context, Result};

use face::Face;
use shape::Shape5;
use utils::{draw_face, print_faces_data};

use rusttype::{Font, Scale};

fn main() -> Result<()> {
    let args = args::parse();

    let image = image::open(&args.input).context("Failed to load image file.")?;

    let (mut detector, shaper, mut localizer) = init::initialize(&args, &image)?;

    let gray = image.to_owned().into_luma8();

    let faces: Vec<Face> = detector
        .detect(&gray)
        .iter()
        .map(|d| {
            let roi = *d.region();

            let shape = shaper.shape(&gray, roi.into());

            let (left_eye_roi, right_eye_roi) = Shape5::find_eyes_roi(&shape);
            let left_pupil = localizer.localize(&gray, left_eye_roi);
            let right_pupil = localizer.localize(&gray, right_eye_roi);

            Face {
                region: roi.into(),
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

    let height = 12.0;
    let scale = Scale {
        x: height,
        y: height,
    };
    let font = load_font()?;

    for face in faces.iter() {
        draw_face(&mut rgb, &face, &font, scale);
    }

    rgb.save(args.output).context("Cannot write output image")?;

    Ok(())
}

fn load_font<'a>() -> Result<Font<'a>> {
    Font::try_from_bytes(include_bytes!("../../assets/DejaVuSansDigits.ttf"))
        .ok_or(anyhow!("Cannot load font."))
}
