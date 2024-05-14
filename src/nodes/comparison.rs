use image::{GenericImageView, Luma};
use nalgebra::Point2;

use pixelutil_image::clamp_pixel_unchecked;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ComparisonNode(pub Point2<i8>, pub Point2<i8>);

impl From<[i8; 4]> for ComparisonNode {
    #[inline]
    fn from(data: [i8; 4]) -> Self {
        let [y0, x0, y1, x1] = data;
        Self(Point2::new(x0, y0), Point2::new(x1, y1))
    }
}

impl From<[u8; 4]> for ComparisonNode {
    #[inline]
    fn from(data: [u8; 4]) -> Self {
        data.map(|value| i8::from_le_bytes(value.to_le_bytes()))
            .into()
    }
}

impl From<ComparisonNode> for [i8; 4] {
    #[inline]
    fn from(node: ComparisonNode) -> [i8; 4] {
        [node.0.y, node.0.x, node.1.y, node.1.x]
    }
}

impl From<ComparisonNode> for [u8; 4] {
    #[inline]
    fn from(node: ComparisonNode) -> [u8; 4] {
        [
            node.0.y.to_le_bytes()[0],
            node.0.x.to_le_bytes()[0],
            node.1.y.to_le_bytes()[0],
            node.1.x.to_le_bytes()[0],
        ]
    }
}

impl ComparisonNode {
    #[inline]
    pub fn bintest<I: GenericImageView<Pixel = Luma<u8>>>(
        &self,
        image: &I,
        point: Point2<i32>,
        size: u32,
    ) -> bool {
        let p0 = transform(point, size, self.0.cast());
        let p1 = transform(point, size, self.1.cast());

        let lum0 = unsafe { clamp_pixel_unchecked(image, p0.x, p0.y) }.0[0];
        let lum1 = unsafe { clamp_pixel_unchecked(image, p1.x, p1.y) }.0[0];

        lum0 > lum1
    }
}

#[allow(dead_code)]
const SCALE: i32 = u8::MAX as i32 + 1;
const SHIFT: i32 = 8;

#[allow(dead_code)]
#[inline]
fn na_transform(i: Point2<i32>, s: u32, n: Point2<i32>) -> Point2<i32> {
    (i * SCALE + n.coords * (s as i32)) / SCALE
}

#[inline]
fn transform(i: Point2<i32>, s: u32, n: Point2<i32>) -> Point2<i32> {
    let (x, y) = original_transform(i.x, i.y, s as i32, n.x, n.y);
    Point2::new(x, y)
}

#[allow(dead_code)]
#[inline]
fn original_transform(ix: i32, iy: i32, s: i32, nx: i32, ny: i32) -> (i32, i32) {
    let x = ((ix << SHIFT) + nx * s) >> SHIFT;
    let y = ((iy << SHIFT) + ny * s) >> SHIFT;
    (x, y)
}

#[cfg(test)]
mod tests {
    use image::{GrayImage, Luma};

    use super::*;

    #[test]
    fn test_comparison_node_from_into() {
        let data: [i8; 4] = [-128, 42, -34, 127];
        let buf: [u8; 4] = ComparisonNode::from(data.clone()).into();
        let out: [i8; 4] = ComparisonNode::from(buf).into();
        assert_eq!(data, out);
    }

    #[test]
    fn test_original_transform() {
        let (ix, iy, s) = (100, 150, 50);
        let (nx, ny) = (42, -34);
        assert_eq!(original_transform(ix, iy, s, nx, ny), (108, 143));
    }

    #[test]
    fn test_na_transform() {
        let i = Point2::new(100, 150);
        let p = Point2::new(42, -34);
        let s = 50;
        assert_eq!(na_transform(i, s, p), transform(i, s, p));
    }

    #[test]
    fn test_comparison_node_bintest() {
        let node = ComparisonNode::from([i8::MAX, i8::MAX, i8::MIN, i8::MIN]);

        let size = 255;
        let mut image = GrayImage::new(size, size);
        image.put_pixel(0, 0, Luma::from([42u8]));
        image.put_pixel(size - 1, size - 1, Luma::from([255u8]));

        let point = Point2::new(size / 2 + 1, size / 2 + 1);
        assert!(node.bintest(&image, point.cast(), size));
    }
}
