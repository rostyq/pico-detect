
use super::detection::Detection;
use super::region::Region;
use super::square::Square;
use super::iou::intersection_over_union;

pub struct Clusterizer {
    intersection_threshold: f32,
    score_threshold: f32,
    buffer: Vec<Detection<Square>>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClusterizerBuilder {
    pub intersection_threshold: Option<f32>,
    pub score_threshold: Option<f32>,
}

impl ClusterizerBuilder {
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

    pub fn with_intersection_threshold(mut self, value: f32) -> Self {
        self.intersection_threshold = Some(value);
        self
    }

    pub fn with_score_threshold(mut self, value: f32) -> Self {
        self.score_threshold = Some(value);
        self
    }
}

impl Clusterizer {
    pub fn builder() -> ClusterizerBuilder {
        Default::default()
    }

    #[inline]
    pub fn push(&mut self, detection: Detection<Square>) {
        self.buffer.push(detection);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    #[inline]
    pub fn clusterize_mut(&mut self, output: &mut Vec<Detection<Square>>) -> usize {
        self.buffer
            .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        let mut assignments = vec![false; self.buffer.len()];
        let mut clusters: usize = 0;

        for (i, det1) in self.buffer.iter().enumerate() {
            if assignments[i] {
                continue;
            } else {
                assignments[i] = true;
            }

            let mut left = det1.region.left();
            let mut top = det1.region.top();

            let mut size = det1.region.size();

            let mut score = det1.score;
            let mut count: usize = 1;

            for (det2, j) in self.buffer[(i + 1)..].iter().zip((i + 1)..) {
                if let Some(value) =
                    intersection_over_union(det1.region.to_owned(), det2.region.to_owned())
                {
                    if value > self.intersection_threshold {
                        assignments[j] = true;

                        left += det2.region.left();
                        top += det2.region.top();

                        size += det2.region.size();

                        score += det2.score * value;
                        count += 1;
                    }
                }
            }

            if score < self.score_threshold {
                continue;
            }

            if count > 1 {
                let count = count as f32;

                left = (left as f32 / count) as i64;
                top = (top as f32 / count) as i64;

                size = (size as f32 / count) as u32;
            }

            let region = Square::new(left, top, size);
            output.push(Detection::new(region, score));
            clusters += 1;
        }
        clusters
    }

    /// Same as `clusterize_mut` but creates new cluster storage.
    #[inline]
    pub fn clusterize(&mut self) -> Vec<Detection<Square>> {
        let mut output = Vec::with_capacity(self.buffer.len());
        self.clusterize_mut(&mut output);
        output
    }
}