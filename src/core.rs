use image::GrayImage;
use na::geometry::{Similarity2, Translation2, UnitComplex};
use na::{Point2, Point3, RealField};
use std::cmp;

pub trait Bintest<N: RealField> {
    fn bintest(&self, image: &GrayImage, transform: &Similarity2<N>) -> bool;
}

#[derive(new)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Leaf {
    x: i8,
    y: i8,
}

impl Leaf {
    pub const SCALE: f32 = u8::MAX as f32;

    pub fn point(&self) -> Point2<f32> {
        Point2::new(self.x as f32, self.y as f32)
    }

    pub fn apply_transform(&self, transform: &Similarity2<f32>) -> Point2<f32> {
        transform.transform_point(&self.point())
    }
}

#[cfg(test)]
impl PartialEq for Leaf {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ComparisonNode(Leaf, Leaf);

impl ComparisonNode {
    pub fn new(data: [i8; 4]) -> Self {
        let [y0, x0, y1, x1] = data;
        Self(Leaf::new(x0, y0), Leaf::new(x1, y1))
    }

    pub fn from_buffer(buf: &[u8; 4]) -> Self {
        let mut data = [0i8; 4];
        for (value, byte) in data.iter_mut().zip(buf.iter()) {
            *value = i8::from_le_bytes(byte.to_le_bytes());
        }
        Self::new(data)
    }
}

#[cfg(test)]
impl PartialEq for ComparisonNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Bintest<f32> for ComparisonNode {
    fn bintest(&self, image: &GrayImage, transform: &Similarity2<f32>) -> bool {
        let p0 = self.0.apply_transform(&transform);
        let p1 = self.1.apply_transform(&transform);

        let lum0 = get_safe_luminance(image, &p0);
        let lum1 = get_safe_luminance(image, &p1);
        lum0 > lum1
    }
}

pub fn create_leaf_transform(point: &Point3<f32>) -> Similarity2<f32> {
    Similarity2::from_parts(
        Translation2::new(point.x, point.y),
        UnitComplex::identity(),
        point.z / Leaf::SCALE,
    )
}

fn get_safe_luminance(image: &GrayImage, point: &Point2<f32>) -> u8 {
    let x = cmp::min(point.x.round() as u32, image.width() - 1);
    let y = cmp::min(point.y.round() as u32, image.height() - 1);
    image.get_pixel(x, y).0[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> GrayImage {
        use image::Luma;
        let mut image = GrayImage::new(width, height);
        image.put_pixel(0, 0, Luma::from([42u8]));
        image.put_pixel(width - 1, height - 1, Luma::from([255u8]));
        image
    }

    #[test]
    fn apply_leaf_transformation() {
        let leaf = Leaf::new(-42, 34);
        let roi = Point3::new(100f32, 100f32, 50f32);

        let test_x = leaf.point().x * roi.z + roi.x;
        let test_y = leaf.point().y * roi.z + roi.y;

        let transform = create_leaf_transform(&roi);
        let point = leaf.apply_transform(&transform);

        abs_diff_eq!(point.x, test_x);
        abs_diff_eq!(point.y, test_y);
    }

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
            let lum = get_safe_luminance(&image, &point);
            assert_eq!(lum, test_lum);
        }
    }

    #[test]
    fn bintest_image_edges() {
        let (width, height) = (255, 255);
        let image = create_test_image(width, height);
        let node = ComparisonNode::new([i8::MAX, i8::MAX, i8::MIN, i8::MIN]);

        let point = Point3::new(
            (width as f32) / 2.0,
            (height as f32) / 2.0,
            width as f32,
        );
        let transform = create_leaf_transform(&point);
        let result = node.bintest(&image, &transform);
        assert!(result);
    }
}
