use std::convert::TryInto;
use std::mem;
use std::mem::MaybeUninit;

use nalgebra::Point2;

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
        let mut out: [MaybeUninit<i8>; 4] = unsafe {
            mem::MaybeUninit::uninit().assume_init()
        };

        for (pos, o) in data.iter().zip(out.iter_mut()) {
            *unsafe { o.assume_init_mut() } = i8::from_le_bytes(pos.to_le_bytes());
        }

        Self::from(
            unsafe {
                mem::transmute::<_, [i8; 4]>(out)
            }
        )
    }
}

impl From<ComparisonNode> for [i8; 4] {
    fn from(node: ComparisonNode) -> [i8; 4] {
        [node.0.y, node.0.x, node.1.y, node.1.x]
    }
}

impl From<ComparisonNode> for [u8; 4] {
    fn from(node: ComparisonNode) -> [u8; 4] {
        [
            node.0.y.to_le_bytes()[0],
            node.0.x.to_le_bytes()[0],
            node.1.y.to_le_bytes()[0],
            node.1.x.to_le_bytes()[0],
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

impl From<ThresholdNode> for [u8; 10] {
    fn from(node: ThresholdNode) -> Self {
        let idx0 = (node.idx.0 as u32).to_be_bytes(); // 4 bytes
        let idx1 = (node.idx.1 as u32).to_be_bytes(); // 4 bytes
        let threshold = node.threshold.to_be_bytes(); // 2 bytes
                                                                // = 10 bytes

        let vals =
            idx0.iter()
                .chain(idx1.iter())
                .chain(threshold.iter());

        let mut out: [MaybeUninit<u8>; 10] = unsafe {
            mem::MaybeUninit::uninit().assume_init()
        };

        for (pos, o) in vals.zip(out.iter_mut()) {
            *unsafe { o.assume_init_mut() } = *pos;
        }

        unsafe {
            mem::transmute::<_, [u8; 10]>(out)
        }
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
