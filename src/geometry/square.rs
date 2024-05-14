use imageproc::rect::Rect;

use crate::traits::region::Region;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Square {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) size: u32,
}

impl Square {
    #[inline]
    pub fn at(x: i32, y: i32) -> Self {
        Self {
            left: x,
            top: y,
            size: 1,
        }
    }

    #[inline]
    pub fn of_size(mut self, value: u32) -> Self {
        self.size = value;
        self
    }

    #[inline]
    pub fn from_region<T: Region>(value: T) -> Result<Self, String> {
        if value.is_square() {
            Ok(Self {
                left: value.left(),
                top: value.top(),
                size: value.width(),
            })
        } else {
            Err("Region is not a square".into())
        }
    }

    #[inline]
    pub fn new(left: i32, top: i32, size: u32) -> Self {
        Self { left, top, size }
    }

    #[inline]
    pub fn size(&self) -> u32 {
        self.size
    }
}

impl Region for Square {
    #[inline]
    fn left(&self) -> i32 {
        self.left
    }

    #[inline]
    fn top(&self) -> i32 {
        self.top
    }

    #[inline]
    fn width(&self) -> u32 {
        self.size
    }

    #[inline]
    fn height(&self) -> u32 {
        self.size
    }

    #[inline]
    fn is_square(&self) -> bool {
        true
    }
}

impl From<(i32, i32, u32)> for Square {
    fn from(value: (i32, i32, u32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl From<Square> for Rect {
    fn from(value: Square) -> Self {
        Self::at(value.left, value.top).of_size(value.size, value.size)
    }
}
