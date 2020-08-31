extern crate image;
extern crate nalgebra as na;
extern crate rand;
extern crate rand_xorshift;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[macro_use]
extern crate derive_new;

mod core;
mod localizer;
mod detector;

pub use localizer::Localizer;
pub use detector::{Detector, CascadeParameters, Detection};
pub use crate::core::create_xorshift_rng;

pub mod test_utils;
