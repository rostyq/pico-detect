use nalgebra::Point2;

pub trait Region {
    fn left(&self) -> i32;
    fn top(&self) -> i32;
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    #[inline]
    fn right(&self) -> i32 {
        self.left() + (self.width() - 1) as i32
    }

    #[inline]
    fn bottom(&self) -> i32 {
        self.top() + (self.height() - 1) as i32
    }

    #[inline]
    fn is_square(&self) -> bool {
        self.width() == self.height()
    }

    #[inline]
    fn contains(&self, x: i32, y: i32) -> bool {
        self.left() <= x && self.top() <= y && self.right() >= x && self.bottom() >= y
    }

    #[inline]
    fn center(&self) -> Point2<i32> {
        Point2::new(
            self.left() + (self.width() / 2 + 1) as i32,
            self.top() + (self.height() / 2 + 1) as i32,
        )
    }

    #[inline]
    fn top_left(&self) -> Point2<i32> {
        Point2::new(self.left(), self.top())
    }

    #[inline]
    fn square(&self) -> u32 {
        self.width() * self.height()
    }
}
