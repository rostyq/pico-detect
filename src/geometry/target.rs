use imageproc::rect::Rect;
use nalgebra::Point2;

use crate::traits::Region;

use super::Square;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub(crate) point: Point2<f32>,
    pub(crate) size: f32,
}

impl Target {
    #[inline]
    pub fn new(x: f32, y: f32, s: f32) -> Self {
        Self {
            point: Point2::new(x, y),
            size: s,
        }
    }

    #[inline]
    pub fn size(&self) -> f32 {
        self.size
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.point.x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.point.y
    }

    #[inline]
    pub fn point(&self) -> &Point2<f32> {
        &self.point
    }
}

impl Region for Target {
    #[inline]
    fn left(&self) -> i32 {
        (self.point.x - self.size / 2.0) as i32
    }

    #[inline]
    fn top(&self) -> i32 {
        (self.point.y - self.size / 2.0) as i32
    }

    #[inline]
    fn width(&self) -> u32 {
        self.size as u32
    }

    #[inline]
    fn height(&self) -> u32 {
        self.size as u32
    }

    #[inline]
    fn is_square(&self) -> bool {
        true
    }

    #[inline]
    fn center(&self) -> Point2<i32> {
        Point2::new(self.x() as i32, self.y() as i32)
    }
}

impl From<Target> for Rect {
    #[inline]
    fn from(value: Target) -> Self {
        Self::at(value.left(), value.top()).of_size(value.width(), value.height())
    }
}

impl From<Target> for Square {
    #[inline]
    fn from(value: Target) -> Self {
        Self::new(value.left(), value.top(), value.size() as u32)
    }
}

impl From<Square> for Target {
    #[inline]
    fn from(value: Square) -> Self {
        Self {
            point: value.center().cast(),
            size: value.size() as f32,
        }
    }
}
