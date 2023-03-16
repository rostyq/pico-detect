use pico_detect::utils::{Square, Point2};

#[derive(Clone, Debug)]
pub struct Face {
    pub score: f32,
    pub region: Square,
    pub shape: Vec<Point2<f32>>,
    pub pupils: (Point2<i64>, Point2<i64>),
}
