extern crate image;
extern crate imageproc;
extern crate nalgebra;
extern crate rand;
extern crate rand_xorshift;
extern crate similarity_least_squares;

pub use image::{GenericImageView, Luma};

#[cfg(test)]
#[macro_use]
extern crate approx;

mod nodes;
pub mod utils;

mod detector;
mod localizer;
mod shaper;

pub use detector::{Detector, MultiscaleDetector, MultiscaleDetectorBuilder};
pub use localizer::{Localizer, PerturbatingLocalizer, PerturbatingLocalizerBuilder};
pub use shaper::Shaper;
