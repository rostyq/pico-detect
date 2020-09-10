use na::{Point2, Vector3, MatrixMN, Similarity2};
use na::{U2, Dynamic};
use na::RealField;

#[inline]
pub fn scale_and_translate_fast(point: &Point2<i8>, transform: &Vector3<i32>) -> Point2<u32> {
    let x = (((transform.x << 8) + (point.x as i32) * transform.z) >> 8) as u32;
    let y = (((transform.y << 8) + (point.y as i32) * transform.z) >> 8) as u32;
    Point2::new(x, y)
}

#[inline]
pub fn find_similarity<T>(from: &MatrixMN<T, U2, Dynamic>, to: &MatrixMN<T, U2, Dynamic>) -> Similarity2<T>
    where 
        T: RealField,
{
    todo!();

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_scale_and_translate() {
        let point = Point2::new(42i8, -34i8);
        let transform = Vector3::new(100i32, 150i32, 50i32);
        assert_eq!(
            scale_and_translate_fast(&point, &transform),
            Point2::new(108u32, 143u32)
        );
    }

    #[test]
    fn check_find_similarity() {

    }
}
