use std::convert::TryInto;
use std::mem::{transmute, MaybeUninit};

#[derive(Debug, Clone, Copy)]
pub struct ThresholdNode {
    pub idx: (usize, usize),
    pub threshold: i16,
}

impl From<[u8; 10]> for ThresholdNode {
    #[inline]
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
    #[inline]
    fn from(node: ThresholdNode) -> Self {
        let idx0 = (node.idx.0 as u32).to_be_bytes(); // 4 bytes
        let idx1 = (node.idx.1 as u32).to_be_bytes(); // 4 bytes
        let threshold = node.threshold.to_be_bytes(); // 2 bytes
                                                      // = 10 bytes

        let vals = idx0.iter().chain(idx1.iter()).chain(threshold.iter());

        let mut out: [MaybeUninit<u8>; 10] = unsafe { MaybeUninit::uninit().assume_init() };

        for (pos, o) in vals.zip(out.iter_mut()) {
            *unsafe { o.assume_init_mut() } = *pos;
        }

        unsafe { transmute::<_, [u8; 10]>(out) }
    }
}

impl ThresholdNode {
    #[inline(always)]
    fn get_value(features: &[u8], index: usize) -> i16 {
        (unsafe { *features.get_unchecked(index) }) as i16
    }

    #[inline]
    pub fn bintest(&self, features: &[u8]) -> bool {
        let diff = Self::get_value(features, self.idx.0) - Self::get_value(features, self.idx.1);
        self.threshold > diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_node_from_into() {
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

    #[test]
    fn test_threshold_node_bintest() {
        let mut node = ThresholdNode {
            idx: (0, 1),
            threshold: 42,
        };

        assert!(node.bintest(&[42, 1]));
        assert!(node.bintest(&[0, 1]));
        assert!(!node.bintest(&[43, 1]));

        node.threshold = -42;

        assert!(node.bintest(&[0, 43]));
        assert!(!node.bintest(&[0, 1]));
    }
}
