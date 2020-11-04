use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};

use super::bintest::ImageBintest;
use super::geometry::ISimilarity2;
use super::node::ComparisonNode;

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

impl Detector {
    /// Estimate detection score for the region of interest.
    ///
    /// ### Arguments
    ///
    /// * `image` -- target image;
    /// * `roi` -- region of interest:
    ///   - `scaling` -- region size;
    ///   - `translation` -- region center.
    ///
    /// ### Returns
    ///
    /// * `Some(f32)` passed region is an object with score;
    /// * `None` -- if passed region is not an object.
    #[inline]
    pub fn classify<I>(&self, image: &I, roi: ISimilarity2) -> Option<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut result = 0.0f32;

        for tree in self.trees.iter() {
            let idx = (0..self.depth).fold(1, |idx, _| {
                2 * idx + !tree.nodes[idx].bintest(image, &roi) as usize
            });
            let lutidx = idx - self.dsize;
            result += tree.predictions[lutidx];

            if result < tree.threshold {
                return None;
            }
        }
        Some(result - self.trees.last().unwrap().threshold)
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
            nodes.push(ComparisonNode::from([0i8, 0i8, 0i8, 0i8]));

            for _ in 0..tree_size {
                readable.read_exact(&mut buffer)?;
                nodes.push(ComparisonNode::from(buffer));
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

    #[test]
    fn check_face_detector_model_parsing() {
        let facefinder =
            Detector::from_readable(include_bytes!("../models/facefinder").to_vec().as_slice())
                .expect("parsing failed");
        assert_eq!(6, facefinder.depth);
        assert_eq!(468, facefinder.trees.len());

        let second_node = ComparisonNode::from([-17i8, 36i8, -55i8, 7i8]);
        let last_node = ComparisonNode::from([-26i8, -84i8, -48i8, 0i8]);
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
}
