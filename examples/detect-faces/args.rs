use anyhow::{Context, Result};
use clap::Parser;
use image::DynamicImage;
use pico_detect::{
    clusterize::Clusterizer, multiscale::Multiscaler, DetectMultiscale, Detector,
    LocalizePerturbate, Localizer, Padding, Shaper,
};
use std::path::PathBuf;

use crate::{load_model, localizer, shaper};

#[derive(Parser, Debug)]
#[command(author, version, about = "CLI human face detection using PICO models.")]
pub struct Args {
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: PathBuf,

    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    #[arg(long = "min", default_value_t = 100)]
    pub min_size: u32,

    #[arg(long = "max")]
    pub max_size: Option<u32>,

    #[arg(long = "scale", default_value_t = 1.1)]
    pub scale_factor: f32,

    #[arg(long = "shift", default_value_t = 0.05)]
    pub shift_factor: f32,

    #[arg(long = "top", default_value_t = 0)]
    pub top_padding: i32,

    #[arg(long = "left", default_value_t = 0)]
    pub left_padding: i32,

    #[arg(long = "bottom", default_value_t = 0)]
    pub bottom_padding: i32,

    #[arg(long = "right", default_value_t = 0)]
    pub right_padding: i32,

    #[arg(long = "intersect", default_value_t = 0.2)]
    pub intersection_threshold: f32,

    #[arg(long = "score", default_value_t = 0.0)]
    pub score_threshold: f32,

    #[arg(long = "perturbs", default_value_t = 15)]
    pub localizer_perturbs: usize,

    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    pub face_finder: Option<PathBuf>,

    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    pub pupil_localizer: Option<PathBuf>,

    #[arg(long, value_hint = clap::ValueHint::FilePath)]
    pub face_shaper: Option<PathBuf>,

    #[arg(long, value_hint = clap::ValueHint::DirPath)]
    pub models_dir: Option<PathBuf>,
}

impl Args {
    pub fn load_models(&self) -> Result<(Detector, Localizer, Shaper)> {
        Ok((detector!(self), localizer!(self), shaper!(self)))
    }

    pub fn init(&self, image: &DynamicImage) -> Result<(DetectMultiscale, LocalizePerturbate)> {
        Ok((
            DetectMultiscale::builder()
                .multiscaler(Multiscaler::new(
                    self.min_size,
                    self.max_size
                        .unwrap_or_else(|| image.height().min(image.width())),
                    self.scale_factor,
                    self.shift_factor,
                )?)
                .clusterizer(Clusterizer {
                    intersection_threshold: self.intersection_threshold,
                    score_threshold: self.score_threshold,
                })
                .padding(Padding {
                    top: self.top_padding,
                    right: self.right_padding,
                    bottom: self.bottom_padding,
                    left: self.left_padding,
                })
                .build()?,
            LocalizePerturbate::new(self.localizer_perturbs),
        ))
    }
}

pub fn parse() -> Args {
    Args::parse()
}
