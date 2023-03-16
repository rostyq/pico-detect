use imageproc::rect::Rect;
use super::region::Region;

#[derive(Clone, Copy, Debug)]
pub struct Square {
    left: i64,
    top: i64,
    size: u32,
}

impl Square {
    pub fn at(x: i64, y: i64) -> Self {
        Self {
            left: x,
            top: y,
            size: 1,
        }
    }

    pub fn of_size(mut self, value: u32) -> Self {
        self.size = value;
        self
    }

    pub fn from_region<T: Region>(value: T) -> Result<Self, &'static str> {
        if value.is_square() {
            Ok(Self {
                left: value.left(),
                top: value.top(),
                size: value.width(),
            })
        } else {
            Err("Region is not a square")
        }
    }

    pub fn new(left: i64, top: i64, size: u32) -> Self {
        Self { left, top, size }
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

impl Region for Square {
    fn left(&self) -> i64 {
        self.left
    }

    fn top(&self) -> i64 {
        self.top
    }

    fn width(&self) -> u32 {
        self.size
    }

    fn height(&self) -> u32 {
        self.size
    }

    fn is_square(&self) -> bool {
        true
    }
}

impl From<(i64, i64, u32)> for Square {
    fn from(value: (i64, i64, u32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl From<Square> for Rect {
    fn from(value: Square) -> Self {
        Self::at(value.left as i32, value.top as i32).of_size(value.size, value.size)
    }
}