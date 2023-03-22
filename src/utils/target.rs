use imageproc::rect::Rect;
use nalgebra::Point2;

use super::{Detection, Region, Square};


#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub(crate) point: Point2<f32>,
    pub(crate) size: f32,
}

impl Target {
    pub(crate) fn new(x: f32, y: f32, s: f32) -> Self {
        Self {
            point: Point2::new(x, y),
            size: s
        }
    }

    pub(crate) fn detection(self, score: f32) -> Detection<Self> {
        Detection { region: self, score }
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn x(&self) -> f32 {
        self.point.x
    }

    pub fn y(&self) -> f32 {
        self.point.y
    }

    pub fn point(&self) -> &Point2<f32> {
        &self.point
    }
}

impl Region for Target {
    fn left(&self) -> i64 {
        (self.point.x - self.size / 2.0) as i64
    }

    fn top(&self) -> i64 {
        (self.point.y - self.size / 2.0) as i64
    }

    fn width(&self) -> u32 {
        self.size as u32
    }

    fn height(&self) -> u32 {
        self.size as u32
    }

    fn is_square(&self) -> bool {
        true
    }

    fn center(&self) -> (i64, i64) {
        (self.x() as i64, self.y() as i64)
    }
}

impl From<Target> for Rect {
    fn from(value: Target) -> Self {
        Self::at(value.left() as i32, value.top() as i32).of_size(value.width(), value.height())
    }
}

impl From<Target> for Square {
    fn from(value: Target) -> Self {
        Self::new(value.left(), value.top(), value.size() as u32)
    }
}

impl From<Square> for Target {
    fn from(value: Square) -> Self {
        let (x, y) = value.center();
        Self::new(x as f32, y as f32, value.size() as f32)
    }
}