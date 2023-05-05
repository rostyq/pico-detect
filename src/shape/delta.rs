use nalgebra::Vector2;

#[derive(Debug, Clone)]
pub struct ShaperDelta {
    anchor: usize,
    value: Vector2<f32>,
}

impl ShaperDelta {
    #[inline]
    pub fn new(anchor: usize, x: f32, y: f32) -> Self {
        Self {
            anchor,
            value: Vector2::new(x, y),
        }
    }

    #[inline]
    pub fn anchor(&self) -> usize {
        self.anchor
    }

    #[inline]
    pub fn value(&self) -> &Vector2<f32> {
        &self.value
    }
}
