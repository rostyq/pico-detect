pub extern crate image;
pub extern crate imageproc;
pub extern crate nalgebra;
pub extern crate rand;

extern crate similarity_least_squares;

extern crate derive_builder;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod geometry;
mod nodes;
mod traits;

mod detect;
mod localize;
mod shape;

pub use geometry::{Square, Target};

pub use detect::{
    clusterize, multiscale, DetectMultiscale, DetectMultiscaleBuilder,
    DetectMultiscaleBuilderError, Detector, Padding, Detection
};
pub use localize::{perturbate, Localizer, LocalizePerturbate};
pub use shape::Shaper;
pub use traits::Region;
