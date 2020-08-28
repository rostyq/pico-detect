use std::cmp::Ordering;

use image::{GenericImageView, GrayImage};
use na::geometry::{Similarity2, Translation2, UnitComplex};
use na::{Point2, Point3};

use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

#[derive(Debug, PartialEq)]
// #[cfg_attr(debug_assertions, derive(Debug))]
pub struct ComparisonNode {
    left: Point2<i8>,
    right: Point2<i8>,
}

pub trait Bintest<T> {
    fn bintest(&self, image: &GrayImage, transform: &T) -> bool;
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

#[inline]
fn find_lum_by_similarity(
    image: &GrayImage,
    transform: &Similarity2<f32>,
    point: &Point2<i8>,
) -> u8 {
    let point = transform_by_similarity(transform, point);
    image.safe_get_lum(point.x, point.y)
}

#[inline]
fn transform_by_similarity(transform: &Similarity2<f32>, point: &Point2<i8>) -> Point2<u32> {
    let x = transform.isometry.translation.x.round() as i32;
    let y = transform.isometry.translation.y.round() as i32;
    let size = transform.scaling() as i32;

    transform_fast(point, x, y, size)
}

impl Bintest<Similarity2<f32>> for ComparisonNode {
    #[inline]
    fn bintest(&self, image: &GrayImage, transform: &Similarity2<f32>) -> bool {
        let lum0 = find_lum_by_similarity(image, transform, &self.left);
        let lum1 = find_lum_by_similarity(image, transform, &self.right);
        lum0 > lum1
    }
}

#[inline]
fn find_lum_by_usize(
    image: &GrayImage,
    transform: &Point3<usize>,
    point: &Point2<i8>,
) -> u8 {
    let point = transform_by_usize(transform, point);
    image.safe_get_lum(point.x, point.y)
}

#[inline]
fn transform_by_usize(transform: &Point3<usize>, point: &Point2<i8>) -> Point2<u32> {
    let x = transform.x as i32;
    let y = transform.y as i32;
    let size = transform.z as i32;
    transform_fast(point, x, y, size)
}

#[inline]
fn transform_fast(point: &Point2<i8>, x: i32, y: i32, size: i32) -> Point2<u32> {
    let x = (((x * 256) + (point.x as i32) * size) / 256) as u32;
    let y = (((y * 256) + (point.y as i32) * size) / 256) as u32;
    Point2::new(x, y)
}

impl Bintest<Point3<usize>> for ComparisonNode {
    #[inline]
    fn bintest(&self, image: &GrayImage, transform: &Point3<usize>) -> bool {
        let lum0 = find_lum_by_usize(image, transform, &self.left);
        let lum1 = find_lum_by_usize(image, transform, &self.right);
        lum0 > lum1
    }
}

pub fn create_leaf_transform(point: &Point3<f32>) -> Similarity2<f32> {
    Similarity2::from_parts(
        Translation2::new(point.x, point.y),
        UnitComplex::identity(),
        point.z,
    )
}

#[inline]
fn saturate_bound(value: u32, bound: u32) -> u32 {
    match value.cmp(&bound) {
        Ordering::Less => value,
        _ => bound - 1,
    }
}

trait SaturatedGet: GenericImageView {
    fn safe_get_lum(&self, x: u32, y: u32) -> u8;
}

impl SaturatedGet for GrayImage {
    fn safe_get_lum(&self, x: u32, y: u32) -> u8 {
        let x = saturate_bound(x, self.width());
        let y = saturate_bound(y, self.height());
        unsafe { self.unsafe_get_pixel(x, y) }.0[0]
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
            let lum = image.safe_get_lum(point.x as u32, point.y as u32);
            assert_eq!(lum, test_lum);
        }
    }

    #[test]
    fn bintest_image_edges() {
        let (width, height) = (255, 255);
        let image = create_test_image(width, height);
        let node = ComparisonNode::new([i8::MAX, i8::MAX, i8::MIN, i8::MIN]);

        let point = Point3::new((width as f32) / 2.0, (height as f32) / 2.0, width as f32);
        let transform = create_leaf_transform(&point);
        let result = node.bintest(&image, &transform);
        assert!(result);
    }
}
