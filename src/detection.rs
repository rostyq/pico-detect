use std::fmt;

use nalgebra::{Isometry2, Point2, Similarity2};

use super::iou::calculate_iou;

/// Object detection data.
#[derive(new, Debug, Copy, Clone)]
pub struct Detection {
    roi: Similarity2<f32>,
    score: f32,
}

impl fmt::Display for Detection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ point: {}, score: {}}}", self.roi, self.score)
    }
}

impl Detection {
    #[inline]
    pub fn from_components(x: f32, y: f32, size: f32, score: f32) -> Self {
        Self::new(
            Similarity2::from_isometry(Isometry2::translation(x, y), size),
            score,
        )
    }

    /// Detection region center.
    #[inline]
    pub fn center(&self) -> Point2<f32> {
        self.roi.isometry.translation.vector.into()
    }

    /// Detection region size.
    #[inline]
    pub fn size(&self) -> f32 {
        self.roi.scaling()
    }

    /// Detection score.
    #[inline]
    pub fn score(&self) -> f32 {
        self.score
    }

    /// Calculate Intersect over Union.
    #[inline]
    pub fn iou(&self, other: &Self) -> f32 {
        calculate_iou(self.center(), other.center(), self.size(), other.size())
    }

    /// Clusterize detections by intersection over union (IoU) metric.
    ///
    /// ### Arguments
    ///
    /// - `detections` -- mutable collection of detections;
    /// - `threshold` -- if IoU is bigger then a detection is a part of a cluster.
    /// - `clusters` -- mutable collection for new clusters.
    #[inline]
    pub fn clusterize_mut(detections: &mut [Self], threshold: f32, clusters: &mut Vec<Self>) {
        detections.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());
        let mut assignments = vec![false; detections.len()];

        for (i, det1) in detections.iter().enumerate() {
            if assignments[i] {
                continue;
            } else {
                assignments[i] = true;
            }

            let mut tvec = det1.center().coords;
            let mut size = det1.size();
            let mut score = det1.score();
            let mut count = 1usize;
            for (det2, j) in detections[(i + 1)..].iter().zip((i + 1)..) {
                if det1.iou(det2) > threshold {
                    assignments[j] = true;
                    tvec += det2.center().coords;
                    score += det2.score();
                    size += det2.size();
                    count += 1;
                }
            }
            if count > 1 {
                let count = count as f32;
                size /= count;
                tvec.x /= count;
                tvec.y /= count;
            }
            clusters.push(Detection::from_components(tvec.x, tvec.y, size, score));
        }
    }

    /// Same as `clusterize_mut` but creates new cluster storage.
    #[inline]
    pub fn clusterize(detections: &mut [Self], threshold: f32) -> Vec<Self> {
        let mut clusters: Vec<Detection> = Vec::with_capacity(detections.len());
        Self::clusterize_mut(detections, threshold, &mut clusters);
        clusters
    }
}
