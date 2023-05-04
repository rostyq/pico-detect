use nalgebra::Point2;

pub trait Region {
    fn left(&self) -> i64;
    fn top(&self) -> i64;
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    #[inline]
    fn right(&self) -> i64 {
        self.left() + (self.width() - 1) as i64
    }

    #[inline]
    fn bottom(&self) -> i64 {
        self.top() + (self.height() - 1) as i64
    }

    #[inline]
    fn is_square(&self) -> bool {
        self.width() == self.height()
    }

    #[inline]
    fn contains(&self, x: i64, y: i64) -> bool {
        self.left() <= x && self.top() <= y && self.right() >= x && self.bottom() >= y
    }

    #[inline]
    fn center(&self) -> Point2<i64> {
        Point2::new(
            self.left() + (self.width() / 2 + 1) as i64,
            self.top() + (self.height() / 2 + 1) as i64,
        )
    }

    #[inline]
    fn top_left(&self) -> Point2<i64> {
        Point2::new(self.left(), self.top())
    }

    #[inline]
    fn square(&self) -> u32 {
        self.width() * self.height()
    }
}
