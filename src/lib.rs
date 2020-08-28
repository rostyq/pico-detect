extern crate image;
extern crate nalgebra as na;
extern crate rand;
extern crate rand_xorshift;

#[macro_use]
extern crate derive_new;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod core;
mod localizer;
mod detector;

pub use localizer::Localizer;
pub use detector::Detector;
pub use crate::core::create_xorshift_rng;

pub mod test_utils;
