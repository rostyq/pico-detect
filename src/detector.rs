use na::{Point3, Similarity2};

use image::GrayImage;

use crate::core::{create_leaf_transform, Bintest, ComparisonNode};
use std::cmp;
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

impl Detector {
    fn classify_region(&self, image: &GrayImage, transform: &Similarity2<f32>) -> Option<f32> {
        let mut result = 0.0f32;

        for tree in self.trees.iter() {
            let idx = (0..self.depth).fold(1, |idx, _| {
                2 * idx + tree.nodes[idx].bintest(&image, &transform) as usize
            });
            let lutidx = idx.saturating_sub(self.dsize);
            result += tree.predictions[lutidx];

            if result < tree.threshold {
                return None;
            }
        }
        Some(result - self.trees.last().unwrap().threshold)
    }

    pub fn run_cascade(
        &self,
        image: &GrayImage,
        min_size: u32,
        max_size: u32,
        shift_factor: u32,
        scale_factor: f32,
    ) -> Vec<(Point3<f32>, f32)> {
        let mut detections = Vec::with_capacity(self.trees.len());
        let (width, height) = (image.width() as usize, image.height() as usize);
        let mut size = min_size;

        while size <= max_size {
            let step = cmp::max(shift_factor * size, 1u32) as usize;
            let offset = (size / 2 + 1) as usize;

            for y in (offset..(height - offset)).step_by(step) {
                for x in (offset..(width - offset)).step_by(step) {
                    let point = Point3::new(x as f32, y as f32, size as f32);
                    let transform = create_leaf_transform(&point);
                    if let Some(probability) = self.classify_region(&image, &transform) {
                        detections.push((point, probability));
                    }
                }
                size = ((size as f32) * scale_factor) as u32;
            }
        }

        detections
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

        let min_size = 50;
        let max_size = 200;
        let shift_factor = 10;
        let scale_factor = 1.1;
        let detections =
            facefinder.run_cascade(&image, min_size, max_size, shift_factor, scale_factor);

        for (point, probability) in detections {
            println!("point: {}, probability: {}", point, probability);
        }
    }
}
