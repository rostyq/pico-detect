use pico_detect::utils::{Point2, Target};

#[derive(Clone, Debug)]
pub struct Face {
    pub score: f32,
    pub region: Target,
    pub shape: Vec<Point2<f32>>,
    pub pupils: (Point2<f32>, Point2<f32>),
}
