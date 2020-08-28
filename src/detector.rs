use crate::core::ComparisonNode;
use std::io::{Error, ErrorKind, Read};

type Tree = Vec<ComparisonNode>;

pub struct Detector {
    depth: usize,
    dsize: usize,
    trees: Vec<Tree>,
    predictions: Vec<f32>,
    thresholds: Vec<f32>,
}

impl Detector {
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

        let mut trees = Vec::with_capacity(ntrees);
        let mut predictions = Vec::with_capacity(2 * ntrees);
        let mut thresholds = Vec::with_capacity(ntrees);

        for _ in 0..ntrees {
            let mut tree = Vec::with_capacity(tree_size);
            tree.push(ComparisonNode::new([0, 0, 0, 0]));

            for _ in 0..tree_size {
                readable.read_exact(&mut buffer)?;
                tree.push(ComparisonNode::from_buffer(&buffer));
            }
            trees.push(tree);

            for _ in 0..pred_size {
                readable.read_exact(&mut buffer)?;
                predictions.push(f32::from_le_bytes(buffer));
            }

            readable.read_exact(&mut buffer)?;
            thresholds.push(f32::from_le_bytes(buffer));
        }

        Ok(Self {
            depth,
            dsize: pred_size,
            trees,
            predictions,
            thresholds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::load_facefinder_model;

    #[test]
    fn check_face_detector_model_parsing() {
        let facefinder = load_facefinder_model();
        assert_eq!(6, facefinder.depth);
        assert_eq!(468, facefinder.trees.len());
        assert_eq!(468, facefinder.thresholds.len());
        assert_eq!(468 * facefinder.dsize, facefinder.predictions.len());

        let second_node = ComparisonNode::new([-17, 36, -55, 7]);
        let last_node = ComparisonNode::new([-26, -84, -48, 0]);
        assert_eq!(second_node, facefinder.trees[0][1]);
        assert_eq!(
            last_node,
            *facefinder.trees.last().unwrap().last().unwrap(),
        );

        assert_abs_diff_eq!(facefinder.thresholds[0], -0.7550662159919739f32);
        assert_abs_diff_eq!(*facefinder.thresholds.last().unwrap(), -1.9176125526428223f32);

        assert_abs_diff_eq!(facefinder.predictions[0], -0.7820115089416504f32);
        assert_abs_diff_eq!(*facefinder.predictions.last().unwrap(), 0.07058460265398026f32);
    }
}
