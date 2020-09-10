use std::cmp::Ordering;
use std::io;

use image::{GenericImageView, GrayImage};
use na::Point2;

use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

#[derive(Debug, PartialEq)]
pub struct ComparisonNode {
    pub left: Point2<i8>,
    pub right: Point2<i8>,
}

impl ComparisonNode {
    pub fn new(data: [i8; 4]) -> Self {
        let [y0, x0, y1, x1] = data;
        Self {
            left: Point2::new(x0, y0),
            right: Point2::new(x1, y1),
        }
    }

    pub fn from_buffer(buf: &[u8; 4]) -> Self {
        let mut data = [0i8; 4];
        for (value, byte) in data.iter_mut().zip(buf.iter()) {
            *value = i8::from_le_bytes(byte.to_le_bytes());
        }
        Self::new(data)
    }
}

pub struct ThresholdNode {
    pub idx: (usize, usize),
    pub threshold: f32,
}

impl ThresholdNode {
    pub fn from_readable(mut readable: impl io::Read) -> io::Result<Self> {
        let mut buf = [0u8; 4];

        readable.read_exact(&mut buf)?;
        let idx0 = u32::from_ne_bytes(buf) as usize;

        readable.read_exact(&mut buf)?;
        let idx1 = u32::from_ne_bytes(buf) as usize;

        readable.read_exact(&mut buf)?;
        let threshold = f32::from_ne_bytes(buf);

        Ok(Self {
            idx: (idx0, idx1),
            threshold,
        })
    }

    #[allow(dead_code)]
    pub fn bintest(&self, feautures: &[f32]) -> bool {
        let diff = feautures[self.idx.0] - feautures[self.idx.1];
        diff > self.threshold
    }
}

pub trait Bintest<T> {
    fn find_point(transform: &T, point: &Point2<i8>) -> Point2<u32>;

    fn find_lum(image: &GrayImage, transform: &T, point: &Point2<i8>) -> u8;

    fn bintest(&self, image: &GrayImage, transform: &T) -> bool;
}

pub trait SaturatedGet: GenericImageView {
    #[inline]
    fn saturate_bound(value: u32, bound: u32) -> u32 {
        match value.cmp(&bound) {
            Ordering::Less => value,
            _ => bound - 1,
        }
    }

    fn saturated_get_lum(&self, x: u32, y: u32) -> u8;
}

impl SaturatedGet for GrayImage {
    #[inline]
    fn saturated_get_lum(&self, x: u32, y: u32) -> u8 {
        let x = Self::saturate_bound(x, self.width());
        let y = Self::saturate_bound(y, self.height());
        unsafe { self.unsafe_get_pixel(x, y) }.0[0]
    }
}

pub trait SafeGet: GenericImageView {
    fn safe_get_lum(&self, x: u32, y: u32, fallback: u8) -> u8;
}

impl SafeGet for GrayImage {
    #[inline]
    fn safe_get_lum(&self, x: u32, y: u32, fallback: u8) -> u8 {
        if self.in_bounds(x, y) {
            unsafe { self.unsafe_get_pixel(x, y) }.0[0]
        } else {
            fallback
        }
    }
}

pub fn create_xorshift_rng(seed: u64) -> XorShiftRng {
    XorShiftRng::seed_from_u64(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_image;

    #[test]
    fn get_luminance_in_and_out_of_image_bounds() {
        let (width, height) = (640, 480);
        let image = create_test_image(width, height);
        let tests = vec![
            (Point2::new(0f32, 0f32), 42u8),
            (Point2::new(-10f32, -10f32), 42u8),
            (Point2::new((width - 1) as f32, (height - 1) as f32), 255u8),
            (Point2::new(width as f32, height as f32), 255u8),
        ];

        for (point, test_lum) in tests {
            let lum = image.saturated_get_lum(point.x as u32, point.y as u32);
            assert_eq!(lum, test_lum);
        }
    }

    #[test]
    fn compare_node_from_buffer_and_new() {
        let (y0, x0, y1, x1) = (-128i8, 42i8, -34i8, 127i8);
        let node1 = ComparisonNode::new([y0, x0, y1, x1]);
        let node2 = ComparisonNode::from_buffer(&[
            y0.to_le_bytes()[0],
            x0.to_le_bytes()[0],
            y1.to_le_bytes()[0],
            x1.to_le_bytes()[0],
        ]);

        assert_eq!(node1, node2);
    }
}
