use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};
use nalgebra::Point2;

use crate::nodes::ComparisonNode;
use crate::utils::region::Region;
use crate::utils::square::Square;
use crate::utils::detection::Detection;
use crate::utils::clusterizer::{Clusterizer, ClusterizerBuilder};
use crate::utils::multiscaler::{Multiscaler, MultiscalerBuilder};

struct Tree {
    nodes: Vec<ComparisonNode>,
    predictions: Vec<f32>,
    threshold: f32,
}

/// Implements object detection using a cascade of decision tree classifiers.
pub struct Detector {
    depth: usize,
    dsize: usize,
    threshold: f32,
    trees: Vec<Tree>,
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
    pub fn detect<I>(&self, image: &I, roi: Square) -> Option<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut result = 0.0f32;
        let (x, y) = roi.center();
        let point = Point2::new(x, y);

        for tree in self.trees.iter() {
            let idx = (0..self.depth).fold(1, |idx, _| {
                2 * idx + !tree.nodes[idx].bintest(image, point, roi.size()) as usize
            });
            let lutidx = idx - self.dsize;
            result += tree.predictions[lutidx];

            if result < tree.threshold {
                return None;
            }
        }
        Some(result - self.threshold)
    }

    /// Create a detector object from a readable source.
    pub fn load(mut readable: impl Read) -> Result<Self, Error> {
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

        let threshold = trees
            .last()
            .ok_or(Error::new(ErrorKind::Other, "No trees"))?
            .threshold;

        Ok(Self {
            depth,
            dsize: pred_size,
            trees,
            threshold,
        })
    }
}

pub struct MultiscaleDetector {
    pub multiscaler: Multiscaler,
    pub clusterizer: Clusterizer,
    pub model: Detector,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MultiscaleDetectorBuilder {
    pub clusterizer_builder: ClusterizerBuilder,
    pub multiscale_builder: MultiscalerBuilder,
}

impl MultiscaleDetectorBuilder {
    pub fn build(self, model: Detector) -> Result<MultiscaleDetector, &'static str> {
        Ok(MultiscaleDetector {
            multiscaler: self.multiscale_builder.build()?,
            clusterizer: self.clusterizer_builder.build()?,
            model,
        })
    }

    pub fn with_clusterizer_builder(mut self, value: ClusterizerBuilder) -> Self {
        self.clusterizer_builder = value;
        self
    }

    pub fn with_multiscale_builder(mut self, value: MultiscalerBuilder) -> Self {
        self.multiscale_builder = value;
        self
    }

    pub fn map_clusterizer_builder<F: FnOnce(ClusterizerBuilder) -> ClusterizerBuilder>(self, f: F) -> Self {
        self.with_clusterizer_builder(f(self.clusterizer_builder))
    }

    pub fn map_multiscale_builder<F: FnOnce(MultiscalerBuilder) -> MultiscalerBuilder>(self, f: F) -> Self {
        self.with_multiscale_builder(f(self.multiscale_builder))
    }
}

impl MultiscaleDetector {
    pub fn builder() -> MultiscaleDetectorBuilder {
        Default::default()
    }

    #[inline]
    fn gather_detections<I>(&mut self, image: &I)
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        self.clusterizer.clear();

        let detector = &self.model;
        let clusterizer = &mut self.clusterizer;

        self.multiscaler.run(image.width(), image.height(), |s| {
            if let Some(score) = detector.detect(image, s) {
                clusterizer.push(Detection::new(s, score));
            }
        });
    }

    #[inline]
    pub fn detect<I>(&mut self, image: &I) -> Vec<Detection<Square>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        self.gather_detections(image);
        self.clusterizer.clusterize()
    }

    #[inline]
    pub fn detect_mut<I>(&mut self, image: &I, output: &mut Vec<Detection<Square>>)
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        self.gather_detections(image);
        self.clusterizer.clusterize_mut(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_detector_model_loading() {
        let facefinder = Detector::load(include_bytes!("../models/face.detector.bin").to_vec().as_slice())
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
