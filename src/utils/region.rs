pub trait Region {
    fn left(&self) -> i64;
    fn top(&self) -> i64;
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn right(&self) -> i64 {
        self.left() + (self.width() - 1) as i64
    }
    fn bottom(&self) -> i64 {
        self.top() + (self.height() - 1) as i64
    }

    fn is_square(&self) -> bool {
        self.width() == self.height()
    }

    fn contains(&self, x: i64, y: i64) -> bool {
        self.left() <= x && self.top() <= y && self.right() >= x && self.bottom() >= y
    }

    fn center(&self) -> (i64, i64) {
        (
            self.left() + (self.width() / 2 + 1) as i64,
            self.top() + (self.height() / 2 + 1) as i64,
        )
    }

    fn square(&self) -> u32 {
        self.width() * self.height()
    }
}
