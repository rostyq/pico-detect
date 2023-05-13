mod tree;

use std::fmt::Debug;
use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};

use crate::geometry::Square;
use crate::traits::Region;

use super::Detection;

use tree::DetectorTree;

/// Implements object detection using a cascade of decision tree classifiers.
#[derive(Clone)]
pub struct Detector {
    depth: usize,
    dsize: usize,
    threshold: f32,
    forest: Vec<DetectorTree>,
}

impl Debug for Detector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(Detector))
            .field("depth", &self.depth)
            .field("dsize", &self.dsize)
            .field("threshold", &self.threshold)
            .field("trees", &self.forest.len())
            .finish()
    }
}

impl Detector {
    /// Estimate detection score for the rectangular region.
    ///
    /// ### Arguments
    ///
    /// * `image` -- target image;
    ///
    /// ### Returns
    ///
    /// * `Some(f32)` passed region is an object with score;
    /// * `None` -- if passed region is not an object.
    #[inline]
    pub fn classify<I>(&self, image: &I, region: Square) -> Option<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut result = 0.0f32;
        let point = region.center();

        for tree in self.forest.iter() {
            let idx = (0..self.depth).fold(1, |idx, _| {
                2 * idx + !tree.nodes[idx].bintest(image, point, region.size()) as usize
            });
            let lutidx = idx - self.dsize;
            result += tree.predictions[lutidx];

            if result < tree.threshold {
                return None;
            }
        }
        Some(result - self.threshold)
    }

    #[inline]
    pub fn detect<I>(&self, image: &I, region: Square) -> Option<Detection<Square>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        self.classify(image, region)
            .map(|score| Detection { region, score })
    }

    /// Create a detector object from a readable source.
    #[inline]
    pub fn load(mut readable: impl Read) -> Result<Self, Error> {
        let mut buffer: [u8; 4] = [0u8; 4];
        // skip first 8 bytes;
        readable.read_exact(&mut [0; 8])?;

        readable.read_exact(&mut buffer)?;
        let depth = i32::from_le_bytes(buffer) as usize;

        let tree_size: usize = match 2usize.checked_pow(depth as u32) {
            Some(value) => value,
            None => return Err(Error::new(ErrorKind::Other, "depth overflow")),
        };

        readable.read_exact(&mut buffer)?;
        let ntrees = i32::from_le_bytes(buffer) as usize;

        let mut trees: Vec<DetectorTree> = Vec::with_capacity(ntrees);

        for _ in 0..ntrees {
            trees.push(DetectorTree::load(&mut readable, tree_size)?);
        }

        let threshold = trees
            .last()
            .ok_or(Error::new(ErrorKind::Other, "No trees"))?
            .threshold;

        Ok(Self {
            depth,
            dsize: tree_size,
            forest: trees,
            threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::nodes::ComparisonNode;

    use super::*;

    #[test]
    fn test_face_detector_model_loading() {
        let facefinder = dbg!(Detector::load(
            include_bytes!("../../../models/face.detector.bin")
                .to_vec()
                .as_slice(),
        )
        .expect("parsing failed"));

        // for tree in facefinder.forest.iter() {
        //     println!("{:?}", tree);
        // }

        assert_eq!(6, facefinder.depth);
        assert_eq!(468, facefinder.forest.len());

        let second_node = ComparisonNode::from([-17i8, 36i8, -55i8, 7i8]);
        let last_node = ComparisonNode::from([-26i8, -84i8, -48i8, 0i8]);
        assert_eq!(second_node, facefinder.forest[0].nodes[1]);
        assert_eq!(
            last_node,
            *facefinder.forest.last().unwrap().nodes.last().unwrap(),
        );

        assert_abs_diff_eq!(facefinder.forest[0].threshold, -0.7550662159919739f32);
        assert_abs_diff_eq!(
            facefinder.forest.last().unwrap().threshold,
            -1.9176125526428223f32
        );

        assert_abs_diff_eq!(facefinder.forest[0].predictions[0], -0.7820115089416504f32);
        assert_abs_diff_eq!(
            *facefinder
                .forest
                .last()
                .unwrap()
                .predictions
                .last()
                .unwrap(),
            0.07058460265398026f32
        );
    }
}
