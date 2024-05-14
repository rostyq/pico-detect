use std::{fs::File, io::BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use image::{Rgb, RgbImage};
use imageproc::{
    drawing::{draw_filled_circle_mut, draw_text_mut},
    geometry::min_area_rect,
    point::Point,
    rect::Rect,
};
use pico_detect::Shaper;
use ab_glyph::FontRef;


#[derive(Parser, Debug)]
#[command(author, version, about = "Print init points from shaper model.")]
struct Args {
    #[arg(value_hint = clap::ValueHint::FilePath)]
    model_path: PathBuf,

    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    output_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let font = FontRef::try_from_slice(include_bytes!("../assets/DejaVuSansDigits.ttf")).expect("Failed to load font.");
    let file = BufReader::new(File::open(&args.model_path).context("Failed to open model file.")?);
    let shaper = Shaper::load(file).context("Error during model loading.")?;

    println!("i,x,y");
    for (i, point) in shaper.init_points().iter().enumerate() {
        println!("{},{},{}", i, point.x, point.y);
    }

    if let Some(path) = args.output_path {
        let points: Vec<Point<i32>> = shaper
            .init_points()
            .iter()
            .map(|p| p.coords.scale(1000.0))
            .map(|v| Point::new(v.x as i32, v.y as i32))
            // .inspect(|p| println!("{}", p))
            .collect();

        let [tl, _, br, _] = min_area_rect(&points);
        let rect = Rect::at(tl.x, tl.y).of_size((br.x - tl.x) as u32, (br.y - tl.y) as u32);

        let padding = 50;

        let mut image = RgbImage::new(rect.width() + padding * 2, rect.height() + padding * 2);

        let color = Rgb::from([0u8, 255u8, 0u8]);
        let scale = 20.0;
        let radius = 5;

        for (i, point) in points.iter().enumerate() {
            let x = padding as i32 + point.x - rect.left();
            let y = padding as i32 + point.y - rect.top();

            draw_filled_circle_mut(&mut image, (x, y), radius, color);
            draw_text_mut(
                &mut image,
                color,
                x + radius,
                y + radius,
                scale,
                &font,
                &format!("{}", i),
            );
        }

        image.save(path).context("Cannot write output image.")?;
    }

    Ok(())
}
