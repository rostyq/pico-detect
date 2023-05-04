use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use pico_detect::{Square, Detector, Localizer, Shaper};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ModelType {
    Detector,
    Localizer,
    Shaper,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Run PICO model on image.")]
struct Args {
    #[arg(short = 't', long, value_enum)]
    model_type: ModelType,

    #[arg(short = 'm', long, value_hint = clap::ValueHint::FilePath)]
    model_path: PathBuf,

    #[arg(short = 'i', long, value_hint = clap::ValueHint::FilePath)]
    image_path: PathBuf,

    #[arg(long, default_value_t = 0)]
    top: i64,

    #[arg(long, default_value_t = 0)]
    left: i64,

    #[arg(long)]
    size: Option<u32>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let image = image::open(&args.image_path)
        .context("Failed to load image file.")?
        .into_luma8();

    let file = File::open(&args.model_path).context("Failed to open model file.")?;

    let left = args.left;
    let top = args.top;
    let size = args
        .size
        .unwrap_or_else(|| image.height().max(image.width()));

    let square = Square::new(left, top, size);

    match args.model_type {
        ModelType::Detector => {
            let detector = Detector::load(file)?;
            if let Some(score) = detector.classify(&image, square) {
                println!("{}", score);
            }
        }
        ModelType::Localizer => {
            let localizer = Localizer::load(file)?;
            let point = localizer.localize(&image, square.into());
            println!("{},{}", point.x as i64, point.y as i64);
        }
        ModelType::Shaper => {
            let shaper = Shaper::load(file)?;
            let shape = shaper.shape(&image, square.into());
            println!("i,x,y");
            for (i, point) in shape.iter().enumerate() {
                println!("{},{},{}", i, point.x as i64, point.y as i64);
            }
        }
    }

    Ok(())
}
