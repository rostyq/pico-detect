use image::{GenericImageView, Luma};

use super::geometry::ISimilarity2;
use super::node::{ComparisonNode, ThresholdNode};
use super::utils::saturating_get_pixel;

pub trait FeatureBintest<T> {
    fn bintest(&self, features: T) -> bool;
}

pub trait ImageBintest<I> {
    fn bintest(&self, image: &I, transform: &ISimilarity2) -> bool;
}

impl FeatureBintest<&[u8]> for ThresholdNode {
    #[inline]
    fn bintest(&self, features: &[u8]) -> bool {
        let diff = features[self.idx.0] as i16 - features[self.idx.1] as i16;
        self.threshold > diff
    }
}

impl<I> ImageBintest<I> for ComparisonNode
where
    I: GenericImageView<Pixel = Luma<u8>>,
{
    #[inline]
    fn bintest(&self, image: &I, transform: &ISimilarity2) -> bool {
        let p0 = transform.transform_point_i8(self.0);
        let p1 = transform.transform_point_i8(self.1);

        let lum0 = saturating_get_pixel(image, p0.x, p0.y).0[0];
        let lum1 = saturating_get_pixel(image, p1.x, p1.y).0[0];

        lum0 > lum1
    }
}

#[cfg(test)]
mod tests {
    use image::{Luma, GrayImage};

    use super::*;

    #[test]
    fn threshold_bintest() {
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

    #[test]
    fn bintest_image_edges() {
        let (width, height) = (255, 255);
        let mut image = GrayImage::new(width, height);
        image.put_pixel(0, 0, Luma::from([42u8]));
        image.put_pixel(width - 1, height - 1, Luma::from([255u8]));
        let node = ComparisonNode::from([i8::MAX, i8::MAX, i8::MIN, i8::MIN]);

        let transform = ISimilarity2::from_components(
            (width / 2 + 1) as i32,
            (height / 2 + 1) as i32,
            width,
        );
        let result = node.bintest(&image, &transform);
        assert!(result);
    }
}
