use nalgebra::Point2;
use std::convert::TryInto;
use std::mem;

#[derive(Debug, PartialEq)]
pub struct ComparisonNode(pub Point2<i8>, pub Point2<i8>);

impl From<[i8; 4]> for ComparisonNode {
    fn from(data: [i8; 4]) -> Self {
        let [y0, x0, y1, x1] = data;
        Self(Point2::new(x0, y0), Point2::new(x1, y1))
    }
}

impl From<[u8; 4]> for ComparisonNode {
    fn from(data: [u8; 4]) -> Self {
        let mut out: [i8; 4] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        data.iter()
            .zip(out.iter_mut())
            .for_each(|(v, o)| *o = i8::from_le_bytes(v.to_le_bytes()));
        Self::from(out)
    }
}

impl Into<[i8; 4]> for ComparisonNode {
    fn into(self) -> [i8; 4] {
        [self.0.y, self.0.x, self.1.y, self.1.x]
    }
}

impl Into<[u8; 4]> for ComparisonNode {
    fn into(self) -> [u8; 4] {
        [
            self.0.y.to_le_bytes()[0],
            self.0.x.to_le_bytes()[0],
            self.1.y.to_le_bytes()[0],
            self.1.x.to_le_bytes()[0],
        ]
    }
}

pub struct ThresholdNode {
    pub idx: (usize, usize),
    pub threshold: i16,
}

impl From<[u8; 10]> for ThresholdNode {
    fn from(data: [u8; 10]) -> Self {
        let idx0 = u32::from_be_bytes(data[0..4].try_into().unwrap()) as usize;
        let idx1 = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
        let threshold = i16::from_be_bytes(data[8..10].try_into().unwrap());
        Self {
            idx: (idx0, idx1),
            threshold,
        }
    }
}

impl Into<[u8; 10]> for ThresholdNode {
    fn into(self) -> [u8; 10] {
        let mut out: [u8; 10] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let idx0 = (self.idx.0 as u32).to_be_bytes();
        let idx1 = (self.idx.1 as u32).to_be_bytes();
        let threshold = self.threshold.to_be_bytes();
        idx0.iter()
            .chain(idx1.iter())
            .chain(threshold.iter())
            .zip(out.iter_mut())
            .for_each(|(v, o)| *o = *v);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_node_convert_back_and_forth() {
        let data: [i8; 4] = [-128, 42, -34, 127];
        let buf: [u8; 4] = ComparisonNode::from(data.clone()).into();
        let out: [i8; 4] = ComparisonNode::from(buf).into();
        assert_eq!(data, out);
    }

    #[test]
    fn threshold_node_convert_back_and_forth() {
        let test_idx = (1, 2);
        let test_threshold = 2;
        let node = ThresholdNode {
            idx: test_idx,
            threshold: test_threshold,
        };
        let data: [u8; 10] = node.into();
        let ThresholdNode { idx, threshold } = ThresholdNode::from(data);
        assert_eq!(idx, test_idx);
        assert_eq!(threshold, test_threshold);
    }
}
