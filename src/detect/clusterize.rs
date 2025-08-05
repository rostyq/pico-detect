use crate::geometry::{intersection_over_union, Square, Target};
use crate::traits::Region;

use super::detection::Detection;

use nalgebra::Point2;

/// Clustering parameters for object detection results.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clusterizer {
    pub intersection_threshold: f32,
    pub score_threshold: f32,
}

impl Clusterizer {
    /// Set the intersection threshold for clustering.
    #[inline]
    pub fn intersection_threshold(self, value: f32) -> Self {
        Self {
            intersection_threshold: value,
            ..self
        }
    }

    /// Set the score threshold for clustering.
    #[inline]
    pub fn score_threshold(self, value: f32) -> Self {
        Self {
            score_threshold: value,
            ..self
        }
    }

    /// Run clustering on the provided detection data.
    #[inline]
    pub fn clusterize(&self, data: &mut [Detection<Square>], dest: &mut Vec<Detection<Target>>) {
        clusterize(
            data,
            self.intersection_threshold,
            self.score_threshold,
            dest,
        );
    }
}

impl Default for Clusterizer {
    /// Create a default clusterizer with intersection threshold of 0.7
    /// and score threshold of 0.0.
    #[inline]
    fn default() -> Self {
        Self {
            intersection_threshold: 0.7,
            score_threshold: 0.0,
        }
    }
}

/// Clusterize detection results based on intersection and score thresholds.
///
/// ### Arguments
///
/// * `data` -- mutable slice of detection data to clusterize;
/// * `intersection_threshold` -- threshold for intersection over union;
/// * `score_threshold` -- threshold for detection score;
/// * `dest` -- destination vector to store clustered detections.
#[inline]
pub fn clusterize<R: Region + Copy>(
    data: &mut [Detection<R>],
    intersection_threshold: f32,
    score_threshold: f32,
    dest: &mut Vec<Detection<Target>>,
) {
    data.sort_by(|a, b| b.partial_cmp(a).unwrap());

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
            if let Some(value) = intersection_over_union(det1.region, det2.region) {
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

            dest.push(Detection {
                region: Target { point, size },
                score,
            });
        }
    }
}
