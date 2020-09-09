use std::io::{Error, Read};

use std::convert::TryInto;
use na::{MatrixMN, U2, Dynamic, Vector2};

pub type Shape = MatrixMN<f32, U2, Dynamic>;

struct ThresholdNode {
    idx: (usize, usize),
    threshold: f32
}

impl ThresholdNode {
    fn from_readable(mut readable: impl Read) -> Self {
        let mut buf = [0u8; 4*3];
        readable.read_exact(&mut buf);

        let idx0 = u32::from_ne_bytes(buf[0..4].try_into().unwrap()) as usize;
        let idx1 = u32::from_ne_bytes(buf[4..8].try_into().unwrap()) as usize;
        let threshold = f32::from_ne_bytes(buf[8..12].try_into().unwrap());
        Self { idx: (idx0, idx1), threshold }
    }

    fn bintest(&self, feautures: &[f32]) -> bool {
        let diff = feautures[self.idx.0] - feautures[self.idx.1];
        diff > self.threshold
    }
}

struct Tree {
    nodes: Vec<ThresholdNode>,
    shapes: Vec<Shape>,
}

struct Forest {
    trees: Vec<Tree>,
    anchors: Vec<u32>,
    deltas: Vec<Vector2<f32>>,
}

pub struct Shaper {
    initial_shape: Shape,
    forests: Vec<Forest>,
}

impl Shaper {
    pub fn from_readable(mut _readable: impl Read) -> Result<Self, Error> {
        todo!();
        Ok(Self {
            initial_shape: Shape::zeros(5),
            forests: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    #[test]
    fn check_face_landmarks_model_parsing() {
        todo!();
        let _shaper = load_face_landmarks_model();
    }
}
