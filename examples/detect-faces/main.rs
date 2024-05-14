extern crate pico_detect;

#[macro_use]
mod models;

mod args;

mod face;
mod shape;
mod utils;

use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128PlusPlus;

use ab_glyph::FontRef;
use anyhow::{Context, Result};

use face::Face;
use shape::Shape5;
use utils::{draw_face, print_faces_data};

fn main() -> Result<()> {
    let args = args::parse();

    let font = FontRef::try_from_slice(include_bytes!("../../assets/DejaVuSansDigits.ttf"))
        .expect("Failed to load font.");

    let image = image::open(&args.input).context("Failed to load image file.")?;

    let (detector, localizer, shaper) = args.load_models()?;
    let (multiscale, localize) = args.init(&image)?;

    let mut rng = Xoroshiro128PlusPlus::seed_from_u64(42);

    let gray = image.to_owned().into_luma8();

    let faces: Vec<Face> = multiscale
        .run(&detector, &gray)
        .iter()
        .map(|d| {
            let roi = *d.region();

            let shape = shaper.shape(&gray, roi.into());

            let (left_eye_roi, right_eye_roi) = Shape5::find_eyes_roi(&shape);
            let left_pupil = localize.run(&localizer, &mut rng, &gray, left_eye_roi.into());
            let right_pupil = localize.run(&localizer, &mut rng, &gray, right_eye_roi.into());

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

    for face in faces.iter() {
        draw_face(&mut rgb, &face, &font, 12.0);
    }

    rgb.save(args.output).context("Cannot write output image")?;

    Ok(())
}
