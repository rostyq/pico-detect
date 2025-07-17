use nalgebra::Point2;

/// A trait defining a rectangular region with methods to access its properties.
pub trait Region {
    /// Returns the left coordinate of the region.
    fn left(&self) -> i32;
    /// Returns the top coordinate of the region.
    fn top(&self) -> i32;
    /// Returns the width of the region.
    fn width(&self) -> u32;
    /// Returns the height of the region.
    fn height(&self) -> u32;

    /// Returns the right coordinate of the region.
    #[inline]
    fn right(&self) -> i32 {
        self.left() + (self.width() - 1) as i32
    }

    /// Returns the bottom coordinate of the region.
    #[inline]
    fn bottom(&self) -> i32 {
        self.top() + (self.height() - 1) as i32
    }

    /// Checks if the region is square.
    #[inline]
    fn is_square(&self) -> bool {
        self.width() == self.height()
    }

    /// Checks if the region contains a point with coordinates `(x, y)`.
    #[inline]
    fn contains(&self, x: i32, y: i32) -> bool {
        self.left() <= x && self.top() <= y && self.right() >= x && self.bottom() >= y
    }

    /// Returns the center point of the region.
    #[inline]
    fn center(&self) -> Point2<i32> {
        Point2::new(
            self.left() + (self.width() / 2 + 1) as i32,
            self.top() + (self.height() / 2 + 1) as i32,
        )
    }

    /// Returns the top-left corner of the region as a point.
    #[inline]
    fn top_left(&self) -> Point2<i32> {
        Point2::new(self.left(), self.top())
    }

    /// Returns the square area of the region.
    #[inline]
    fn square(&self) -> u32 {
        self.width() * self.height()
    }
}
