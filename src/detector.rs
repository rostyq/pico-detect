use na::Scalar;
use na::{Point2, Point3};

use image::GrayImage;

use crate::core::{Bintest, ComparisonNode};
use std::cmp::{max, min, PartialOrd};
use std::io::{Error, ErrorKind, Read};

struct Tree {
    nodes: Vec<ComparisonNode>,
    predictions: Vec<f32>,
    threshold: f32,
}

pub struct Detector {
    depth: usize,
    dsize: usize,
    trees: Vec<Tree>,
}

#[derive(new, Debug)]
pub struct CascadeParameters {
    pub min_size: usize,
    pub max_size: usize,
    pub shift_factor: f32,
    pub scale_factor: f32,
}

#[derive(Debug)]
pub struct Detection<T: Scalar> {
    pub point: Point3<T>,
    pub score: f32,
}

impl Detection<f32> {
    fn new(x: f32, y: f32, size: f32, score: f32) -> Self {
        Self {
            point: Point3::new(x, y, size),
            score,
        }
    }
}

impl Detection<usize> {
    #[inline]
    fn new(point: Point3<usize>, score: f32) -> Self {
        Self { point, score }
    }
}

impl From<Detection<usize>> for Detection<f32> {
    #[inline]
    fn from(d: Detection<usize>) -> Self {
        Detection::<f32>::new(
            d.point.x as f32,
            d.point.y as f32,
            d.point.z as f32,
            d.score,
        )
    }
}

impl Detector {
    #[inline]
    fn classify_region(&self, image: &GrayImage, transform: &Point3<usize>) -> Option<f32> {
        let mut result = 0.0f32;

        for tree in self.trees.iter() {
            let idx = (0..self.depth).fold(1, |idx, _| {
                2 * idx
                    + unsafe { tree.nodes.get_unchecked(idx) }.bintest(image, transform) as usize
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
    pub fn run_cascade_mut(
        &self,
        image: &GrayImage,
        detections: &mut Vec<Detection<usize>>,
        params: &CascadeParameters,
    ) {
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut size = params.min_size;

        while size <= params.max_size {
            let step = max((params.shift_factor * (size as f32)) as usize, 1);
            let offset = size / 2 + 1;

            for y in (offset..(height - offset)).step_by(step) {
                for x in (offset..(width - offset)).step_by(step) {
                    let point = Point3::new(x, y, size);
                    if let Some(score) = self.classify_region(&image, &point) {
                        detections.push(Detection::<usize>::new(point, score));
                    }
                }
            }
            size = ((size as f32) * params.scale_factor) as usize;
        }
    }

    #[inline]
    pub fn run_cascade(
        &self,
        image: &GrayImage,
        params: &CascadeParameters,
    ) -> Vec<Detection<usize>> {
        let mut detections = Vec::new();
        self.run_cascade_mut(image, &mut detections, params);
        detections
    }

    #[inline]
    pub fn find_clusters(
        &self,
        image: &GrayImage,
        params: &CascadeParameters,
        threshold: f32,
    ) -> Vec<Detection<f32>> {
        let detections = self.run_cascade(image, params);
        Self::cluster_detections(detections, threshold)
    }

    #[inline]
    pub fn cluster_detections(
        mut detections: Vec<Detection<usize>>,
        threshold: f32,
    ) -> Vec<Detection<f32>> {
        detections.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
        let mut assignments = vec![false; detections.len()];
        let mut clusters: Vec<Detection<f32>> = Vec::with_capacity(detections.len());

        for i in 0..detections.len() {
            if assignments[i] {
                continue;
            }

            let mut x = 0usize;
            let mut y = 0usize;
            let mut size = 0usize;
            let mut score = 0f32;
            let mut count = 0usize;
            for j in (i + 1)..detections.len() {
                let (d_i, d_j) = (&detections[i], &detections[j]);
                if calculate_iou(&d_i.point, &d_j.point) > threshold {
                    assignments[i] = true;
                    x += d_i.point.x;
                    y += d_i.point.y;
                    size += d_i.point.z;
                    score += d_i.score;
                    count += 1;
                }
            }

            if count > 0 {
                let n = count as f32;
                clusters.push(Detection::<f32>::new(
                    (x as f32) / n,
                    (y as f32) / n,
                    (size as f32) / n,
                    score,
                ));
            }
        }
        clusters
    }

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
#[inline]
fn roi_to_bbox(p: &Point3<usize>) -> (Point2<usize>, Point2<usize>) {
    let h = p.z / 2;
    (
        Point2::new(p.x.saturating_sub(h), p.y.saturating_sub(h)),
        Point2::new(p.x + h, p.y + h),
    )
}

/// (x, y, size) -> (x0, x1, y0, y1)
#[allow(dead_code)]
fn roi_to_bbox_f(p: &Point3<f32>) -> (Point2<f32>, Point2<f32>) {
    let h = p.z / 2.0;
    (
        Point2::new(p.x - h, p.y - h),
        Point2::new(p.x + h, p.y + h),
    )
}

/// Intersection over Union (IoU)
#[inline]
fn calculate_iou(p0: &Point3<usize>, p1: &Point3<usize>) -> f32 {
    let b0 = roi_to_bbox(p0);
    let b1 = roi_to_bbox(p1);

    let ix = ((min(b0.1.x, b1.1.x) as i32) - (max(b0.0.x, b1.0.x) as i32)).abs() as usize;
    let iy = ((min(b0.1.y, b1.1.y) as i32) - (max(b0.0.y, b1.0.y) as i32)).abs() as usize;

    let inter_square = ix * iy;
    let union_square = (p0.z * p0.z + p1.z * p1.z) - inter_square;

    (inter_square as f32) / (union_square as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{load_facefinder_model, load_test_image};

    #[test]
    fn check_face_detector_model_parsing() {
        let facefinder = load_facefinder_model();
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
    fn check_face_detection() {
        let facefinder = load_facefinder_model();
        let (image, _data) = load_test_image();

        let params = CascadeParameters::new(150, 300, 0.05, 1.05);
        let detections = facefinder.find_clusters(&image, &params, 0.1);

        for (i, detection) in detections.iter().enumerate() {
            let bbox = roi_to_bbox_f(&detection.point);
            println!(
                "{} :: bbox: {}, {}; score: {}",
                i, bbox.0, bbox.1, detection.score
            );
        }

        assert_eq!(detections.len(), 1);
        let detection = &detections[0];

        assert_abs_diff_eq!(
            detection.point,
            Point3::new(289.0, 316.0, 180.0),
            epsilon = 2.0
        );
        assert_abs_diff_eq!(detection.score, 0.6565249);
    }

    #[test]
    fn check_iou() {
        let p0 = Point3::<usize>::new(100, 100, 60);
        let p1 = Point3::<usize>::new(125, 125, 65);
        assert_abs_diff_eq!(calculate_iou(&p0, &p1), 0.21205081);
    }
}
