use core::mem::size_of;
use std::{
    fmt::Debug,
    io::{Error, Read},
};

use crate::nodes::ComparisonNode;

#[derive(Clone)]
pub struct DetectorTree {
    pub(super) threshold: f32,
    pub(super) predictions: Vec<f32>,
    pub(super) nodes: Vec<ComparisonNode>,
}

impl Debug for DetectorTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(DetectorTree))
            .field("threshold", &self.threshold)
            .field("size", &self.predictions.len())
            .finish()
    }
}

impl DetectorTree {
    #[inline]
    fn load_nodes(mut readable: impl Read, count: usize) -> Result<Vec<ComparisonNode>, Error> {
        let mut buffer: [u8; 4] = [0u8; size_of::<ComparisonNode>()];

        let mut nodes = Vec::with_capacity(count);
        nodes.push(ComparisonNode::default());

        for _ in 0..count {
            readable.read_exact(&mut buffer)?;
            nodes.push(ComparisonNode::from(buffer));
        }

        Ok(nodes)
    }

    #[inline]
    fn load_predictions(mut readable: impl Read, count: usize) -> Result<Vec<f32>, Error> {
        let mut buffer: [u8; 4] = [0u8; size_of::<f32>()];

        let mut predictions = Vec::with_capacity(count);

        for _ in 0..count {
            readable.read_exact(&mut buffer)?;
            predictions.push(f32::from_le_bytes(buffer));
        }

        Ok(predictions)
    }

    #[inline]
    pub(super) fn load(
        mut readable: impl Read,
        size: usize,
    ) -> Result<Self, Error> {
        let nodes = Self::load_nodes(readable.by_ref(), size - 1)?;
        let predictions = Self::load_predictions(readable.by_ref(), size)?;

        let mut buffer: [u8; 4] = [0u8; 4];
        readable.read_exact(&mut buffer)?;
        let threshold = f32::from_le_bytes(buffer);

        Ok(DetectorTree {
            nodes,
            predictions,
            threshold,
        })
    }
}
