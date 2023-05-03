
use super::Clusterizer;

#[derive(Clone, Copy, Debug, Default)]
pub struct ClusterizerBuilder {
    pub intersection_threshold: Option<f32>,
    pub score_threshold: Option<f32>,
}

impl ClusterizerBuilder {
    #[inline]
    pub fn build(self) -> Result<Clusterizer, &'static str> {
        let intersection_threshold = match self.intersection_threshold {
            Some(value) => {
                if value.is_normal() & value.is_sign_positive() {
                    value
                } else {
                    return Err("intersection_threshold should be normal and positive")
                }
            },
            None => 0.2,
        };

        let score_threshold = match self.score_threshold {
            Some(value) => {
                if value.is_sign_positive() {
                    value
                } else {
                    return Err("score_threshold should be positive")
                }
            },
            None => 0.0
        };

        Ok(Clusterizer {
            intersection_threshold,
            score_threshold,
            buffer: Default::default()
        })
    }

    #[inline]
    pub fn with_intersection_threshold(mut self, value: f32) -> Self {
        self.intersection_threshold = Some(value);
        self
    }

    #[inline]
    pub fn with_score_threshold(mut self, value: f32) -> Self {
        self.score_threshold = Some(value);
        self
    }
}
