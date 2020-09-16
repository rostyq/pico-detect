use std::io::{Error, ErrorKind, Read};
use std::ops::AddAssign;
use std::{cmp, fmt};

use image::GrayImage;
use na::{Point2, Point3, Vector3};

use super::core::{Bintest, ComparisonNode, SaturatedGet};
use super::geometry::scale_and_translate_fast;

impl Bintest<Point3<u32>> for ComparisonNode {
    #[inline]
    fn find_point(transform: &Point3<u32>, point: &Point2<i8>) -> Point2<u32> {
        scale_and_translate_fast(
            point,
            &Vector3::new(transform.x as i32, transform.y as i32, transform.z as i32),
        )
    }

    #[inline]
    fn find_lum(image: &GrayImage, transform: &Point3<u32>, point: &Point2<i8>) -> u8 {
        let point = Self::find_point(transform, point);
        image.saturated_get_lum(point.x, point.y)
    }

    #[inline]
    fn bintest(&self, image: &GrayImage, transform: &Point3<u32>) -> bool {
        let lum0 = Self::find_lum(image, transform, &self.left);
        let lum1 = Self::find_lum(image, transform, &self.right);
        lum0 <= lum1
    }
}

struct Tree {
    nodes: Vec<ComparisonNode>,
    predictions: Vec<f32>,
    threshold: f32,
}

/// Implements object detection using a cascade of decision tree classifiers.
pub struct Detector {
    depth: usize,
    dsize: usize,
    trees: Vec<Tree>,
}

/// Cascade parameters for `Detector`.
#[derive(new, Debug)]
pub struct CascadeParameters {
    /// Minimum size of an object.
    pub min_size: u32,
    /// Maximum size of an object.
    pub max_size: u32,
    /// How far to move the detection window by fraction of its size: (0..1).
    pub shift_factor: f32,
    /// For multiscale processing: resize the detection window by fraction
    /// of its size when moving to the higher scale: (0..1).
    pub scale_factor: f32,
}

/// Object detection data.
#[derive(Debug, Copy, Clone)]
pub struct Detection {
    /// Region of interest where `x` and `y` center coordinates,
    /// and `z` is a size of the region.
    pub point: Point3<f32>,
    /// Detection score.
    pub score: f32,
}

impl fmt::Display for Detection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ point: {}, score: {}}}", self.point, self.score)
    }
}

impl Detection {
    fn new(x: f32, y: f32, size: f32, score: f32) -> Self {
        Self {
            point: Point3::new(x, y, size),
            score,
        }
    }

    fn scale_mut(&mut self, value: f32) {
        self.point.coords.scale_mut(value);
    }
}

impl AddAssign for Detection {
    fn add_assign(&mut self, rhs: Self) {
        self.point += rhs.point.coords;
        self.score += rhs.score;
    }
}

impl Detector {
    /// Estimate detection score for the region of interest.
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// * `roi` - region of interest:
    ///   - `roi.x` position on image x-axis,
    ///   - `roi.y` position on image y-axis,
    ///   - `roi.z` region size.
    #[inline]
    pub fn classify_region(&self, image: &GrayImage, roi: &Point3<u32>) -> Option<f32> {
        let mut result = 0.0f32;

        for tree in self.trees.iter() {
            let idx = (0..self.depth).fold(1, |idx, _| {
                2 * idx + unsafe { tree.nodes.get_unchecked(idx) }.bintest(image, roi) as usize
            });
            let lutidx = idx.saturating_sub(self.dsize);
            result += tree.predictions[lutidx];

            if result < tree.threshold {
                return None;
            }
        }
        Some(result - self.trees.last().unwrap().threshold)
    }

    #[inline]
    /// Run cascade and push detections to the existing collection.
    pub fn run_cascade_mut(
        &self,
        image: &GrayImage,
        detections: &mut Vec<Detection>,
        params: &CascadeParameters,
    ) {
        let (width, height) = (image.width(), image.height());
        let mut size = params.min_size;

        while size <= params.max_size {
            let sizef = size as f32;
            let step = cmp::max((sizef * params.shift_factor) as usize, 1);
            let offset = size / 2 + 1;

            for y in (offset..(height - offset)).step_by(step) {
                for x in (offset..(width - offset)).step_by(step) {
                    if let Some(score) = self.classify_region(&image, &Point3::new(x, y, size)) {
                        detections.push(Detection::new(x as f32, y as f32, size as f32, score));
                    }
                }
            }
            size = (sizef * params.scale_factor) as u32;
        }
    }

    #[inline]
    /// Run cascade with a new empty detection collection.
    pub fn run_cascade(&self, image: &GrayImage, params: &CascadeParameters) -> Vec<Detection> {
        let mut detections = Vec::new();
        self.run_cascade_mut(image, &mut detections, params);
        detections
    }

    /// Run cascade and clusterize resulted detections using
    /// `Self::cluster_detections`
    #[inline]
    pub fn find_clusters(
        &self,
        image: &GrayImage,
        params: &CascadeParameters,
        threshold: f32,
    ) -> Vec<Detection> {
        let detections = self.run_cascade(image, params);
        Self::cluster_detections(detections, threshold)
    }

    /// Clusterize detections by intersection over union (IoU) metric.
    ///
    /// ### Arguments
    ///
    /// - `detections` -- mutable collection of detections;
    /// - `threshold` -- if IoU is bigger then a detection is a part of a cluster.
    #[inline]
    pub fn cluster_detections(mut detections: Vec<Detection>, threshold: f32) -> Vec<Detection> {
        detections.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        let mut assignments = vec![false; detections.len()];
        let mut clusters: Vec<Detection> = Vec::with_capacity(detections.len());

        for (i, det1) in detections.iter().enumerate() {
            if assignments[i] {
                continue;
            } else {
                assignments[i] = true;
            }

            let (mut cluster, mut count) = (*det1, 1usize);
            for (det2, j) in detections[(i + 1)..].iter().zip((i + 1)..) {
                if calculate_iou(&det1.point, &det2.point) > threshold {
                    assignments[j] = true;
                    cluster += *det2;
                    count += 1;
                }
            }
            if count > 1 {
                cluster.scale_mut((count as f32).recip())
            }
            clusters.push(cluster);
        }
        clusters
    }

    /// Create a detector object from a readable source.
    pub fn from_readable(mut readable: impl Read) -> Result<Self, Error> {
        let mut buffer: [u8; 4] = [0u8; 4];
        // skip first 8 bytes;
        readable.read_exact(&mut [0; 8])?;

        readable.read_exact(&mut buffer)?;
        let depth = i32::from_le_bytes(buffer) as usize;

        let pred_size: usize = match 2usize.checked_pow(depth as u32) {
            Some(value) => value,
            None => return Err(Error::new(ErrorKind::Other, "depth overflow")),
        };
        // first node appended from code
        let tree_size = pred_size - 1;

        readable.read_exact(&mut buffer)?;
        let ntrees = i32::from_le_bytes(buffer) as usize;

        let mut trees: Vec<Tree> = Vec::with_capacity(ntrees);

        for _ in 0..ntrees {
            let mut nodes = Vec::with_capacity(tree_size);
            let mut predictions = Vec::with_capacity(pred_size);
            nodes.push(ComparisonNode::new([0, 0, 0, 0]));

            for _ in 0..tree_size {
                readable.read_exact(&mut buffer)?;
                nodes.push(ComparisonNode::from_buffer(&buffer));
            }

            for _ in 0..pred_size {
                readable.read_exact(&mut buffer)?;
                predictions.push(f32::from_le_bytes(buffer));
            }

            readable.read_exact(&mut buffer)?;
            let threshold = f32::from_le_bytes(buffer);
            trees.push(Tree {
                nodes,
                predictions,
                threshold,
            });
        }

        Ok(Self {
            depth,
            dsize: pred_size,
            trees,
        })
    }
}

/// (x, y, size) -> (x0, x1, y0, y1)
#[allow(dead_code)]
fn roi_to_bbox(p: &Point3<f32>) -> (Point2<f32>, Point2<f32>) {
    let h = p.z / 2.0;
    (Point2::new(p.x - h, p.y - h), Point2::new(p.x + h, p.y + h))
}

/// Intersection over Union (IoU)
#[inline]
fn calculate_iou(p0: &Point3<f32>, p1: &Point3<f32>) -> f32 {
    #[inline]
    fn max(v1: f32, v2: f32) -> f32 {
        if v1 > v2 {
            v1
        } else {
            v2
        }
    }

    #[inline]
    fn min(v1: f32, v2: f32) -> f32 {
        if v1 < v2 {
            v1
        } else {
            v2
        }
    }

    let b0 = roi_to_bbox(p0);
    let b1 = roi_to_bbox(p1);

    let ix = max(0f32, min(b0.1.x, b1.1.x) - max(b0.0.x, b1.0.x));
    let iy = max(0f32, min(b0.1.y, b1.1.y) - max(b0.0.y, b1.0.y));

    let inter_square = ix * iy;
    let union_square = (p0.z * p0.z + p1.z * p1.z) - inter_square;

    inter_square / union_square
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_face_detector_model_parsing() {
        let facefinder =
            Detector::from_readable(include_bytes!("../models/facefinder").to_vec().as_slice())
                .expect("parsing failed");
        assert_eq!(6, facefinder.depth);
        assert_eq!(468, facefinder.trees.len());

        let second_node = ComparisonNode::new([-17, 36, -55, 7]);
        let last_node = ComparisonNode::new([-26, -84, -48, 0]);
        assert_eq!(second_node, facefinder.trees[0].nodes[1]);
        assert_eq!(
            last_node,
            *facefinder.trees.last().unwrap().nodes.last().unwrap(),
        );

        assert_abs_diff_eq!(facefinder.trees[0].threshold, -0.7550662159919739f32);
        assert_abs_diff_eq!(
            facefinder.trees.last().unwrap().threshold,
            -1.9176125526428223f32
        );

        assert_abs_diff_eq!(facefinder.trees[0].predictions[0], -0.7820115089416504f32);
        assert_abs_diff_eq!(
            *facefinder.trees.last().unwrap().predictions.last().unwrap(),
            0.07058460265398026f32
        );
    }

    #[test]
    fn check_iou() {
        let tests = vec![
            (
                Point3::<f32>::new(100.0, 100.0, 50.0),
                Point3::<f32>::new(200.0, 100.0, 50.0),
                0.0,
            ),
            (
                Point3::<f32>::new(100.0, 100.0, 50.0),
                Point3::<f32>::new(100.0, 200.0, 50.0),
                0.0,
            ),
            (
                Point3::<f32>::new(100.0, 100.0, 50.0),
                Point3::<f32>::new(200.0, 200.0, 50.0),
                0.0,
            ),
            (
                Point3::<f32>::new(100.0, 100.0, 50.0),
                Point3::<f32>::new(100.0, 100.0, 50.0),
                1.0,
            ),
            (
                Point3::<f32>::new(100.0, 100.0, 50.0),
                Point3::<f32>::new(125.0, 100.0, 50.0),
                0.3333333,
            ),
            (
                Point3::<f32>::new(100.0, 100.0, 50.0),
                Point3::<f32>::new(100.0, 125.0, 50.0),
                0.3333333,
            ),
            (
                Point3::<f32>::new(100.0, 100.0, 60.0),
                Point3::<f32>::new(125.0, 125.0, 65.0),
                0.21908471,
            ),
        ];

        for (pt1, pt2, iou) in tests.iter() {
            assert_abs_diff_eq!(calculate_iou(pt1, pt2), iou);
        }
    }
}
