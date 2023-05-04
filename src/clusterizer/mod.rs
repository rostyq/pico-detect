use crate::utils::iou::intersection_over_union;
use crate::utils::region::Region;
use crate::utils::target::Target;
use crate::utils::{detection::Detection, Square};

use nalgebra::Point2;

pub struct Clusterizer {
    pub intersection_threshold: f32,
    pub score_threshold: f32,
}

impl Clusterizer {
    #[inline]
    pub fn intersection_threshold(self, value: f32) -> Self {
        Self { intersection_threshold: value, ..self }
    }

    #[inline]
    pub fn score_threshold(self, value: f32) -> Self {
        Self { score_threshold: value, ..self }
    }

    #[inline]
    pub fn push<R: Region>(&mut self, detection: Detection<R>) {
        self.buffer.push(Detection {
            region: Square {
                left: detection.region.left(),
                top: detection.region.top(),
                size: detection.region.width(),
            },
            score: detection.score,
        });
    }

    #[inline]
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    #[inline]
    pub fn clusterize(&mut self) -> Vec<Detection<Target>> {
        let mut output = Vec::with_capacity(self.buffer.len());

        clusterize(
            &mut self.buffer,
            self.intersection_threshold,
            self.score_threshold,
            &mut output,
        );

        output
    }
}

impl Default for Clusterizer {
    #[inline]
    fn default() -> Self {
        Self {
            intersection_threshold: 0.7,
            score_threshold: 0.0,
        }
    }
}

#[inline]
pub fn clusterize<R: Region + Copy>(
    data: &mut [Detection<R>],
    intersection_threshold: f32,
    score_threshold: f32,
    output: &mut Vec<Detection<Target>>,
) {
    data.sort_by(|a, b| b.partial_cmp(&a).unwrap());

    let mut assignments = vec![false; data.len()];

    for (i, det1) in data.iter().enumerate() {
        if assignments[i] {
            continue;
        } else {
            assignments[i] = true;
        }

        let mut point = det1.region.top_left();
        let mut size = det1.region.width();

        let mut score = det1.score;
        let mut count: usize = 1;

        for (det2, j) in data[(i + 1)..].iter().zip((i + 1)..) {
            if let Some(value) = intersection_over_union(*&det1.region, *&det2.region) {
                if value > intersection_threshold {
                    assignments[j] = true;

                    point += det2.region.top_left().coords;
                    size += det2.region.width();

                    score += det2.score * value;
                    count += 1;
                }
            }
        }

        if score > score_threshold {
            let scale = (count as f32).recip();

            let size = (size as f32) * scale;

            let mut point: Point2<f32> = point.cast();

            point.coords.scale_mut(scale);
            point.coords.add_scalar_mut(size / 2.0);

            output.push(Detection {
                region: Target { point, size },
                score,
            });
        }
    }
}
