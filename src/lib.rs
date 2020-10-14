extern crate image;
extern crate nalgebra as na;
extern crate rand;
extern crate rand_xorshift;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[macro_use]
extern crate derive_new;

#[allow(dead_code)]
mod utils;

mod core;
mod geometry;
mod localizer;
mod detector;
mod shaper;

pub use localizer::Localizer;
pub use detector::{Detector, CascadeParameters, Detection};
pub use shaper::Shaper;
pub use crate::core::create_xorshift_rng;
