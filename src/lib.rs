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

mod clusterizer;
mod multiscaler;
mod perturbator;

mod detector;
mod localizer;
mod shaper;

pub use clusterizer::{Clusterizer, ClusterizerBuilder};
pub use multiscaler::{Multiscaler, MultiscalerBuilder};
pub use perturbator::{Perturbator, PerturbatorBuilder};

pub use detector::{Detector, MultiscaleDetector, MultiscaleDetectorBuilder};
pub use localizer::{Localizer, PerturbatingLocalizer, PerturbatingLocalizerBuilder};
pub use shaper::Shaper;
