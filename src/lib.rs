pub extern crate image;
pub extern crate nalgebra;
extern crate rand;

#[macro_use]
extern crate derive_new;

#[cfg(test)]
#[macro_use]
extern crate approx;

mod utils;
mod iou;
mod bintest;
mod node;
mod geometry;
mod detector;
mod detection;
mod multiscale;
mod localizer;
mod shaper;

pub use geometry::ISimilarity2;
pub use detector::Detector;
pub use detection::Detection;
pub use multiscale::MultiScale;
pub use localizer::Localizer;
pub use imageproc::rect::{Rect, RectPosition};
pub use shaper::Shaper;
