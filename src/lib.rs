extern crate image;
extern crate nalgebra as na;
extern crate rand;
extern crate rand_xorshift;

#[macro_use]
extern crate derive_new;

#[cfg(test)]
#[macro_use]
extern crate approx;

pub mod core;
pub mod localizer;
pub mod models;
