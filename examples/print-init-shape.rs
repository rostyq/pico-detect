use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use pico_detect::Shaper;

#[derive(Parser, Debug)]
#[command(author, version, about = "Print init points from shaper model.")]
struct Args {
    #[arg(value_hint = clap::ValueHint::FilePath)]
    model_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = File::open(&args.model_path).context("Failed to open model file.")?;
    let shaper = Shaper::load(file)?;

    println!("i,x,y");
    for (i, point) in shaper.init_points().iter().enumerate() {
        println!("{},{},{}", i, point.x, point.y);
    }

    Ok(())
}
