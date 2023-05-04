use std::cmp::{min, max};

use crate::traits::Region;

#[inline]
pub fn intersection_over_union<R: Region>(r1: R, r2: R) -> Option<f32> {
    let left = max(r1.left(), r2.left());
    let top = max(r1.top(), r2.top());
    let right = min(r1.right(), r2.right());
    let bottom = min(r1.bottom(), r2.bottom());

    if right < left || bottom < top {
        return None;
    }

    let width = (right - left) as u32 + 1;
    let height = (bottom - top) as u32 + 1;

    let inter_square = width * height;
    let union_square = (r1.square() + r2.square()) - inter_square;

    Some(inter_square as f32 / union_square as f32)
}

#[cfg(test)]
mod tests {
    use crate::geometry::Square;

    use super::*;

    #[test]
    fn test_intersection_over_union() {
        let tests: Vec<(Square, Square, Option<f32>)> = vec![
            (
                Square::at(0, 0).of_size(1),
                Square::at(0, 0).of_size(1),
                Some(1.0),
            ),
            (
                Square::at(0, 0).of_size(2),
                Square::at(1, 1).of_size(2),
                Some(1.0 / (2 * 2 * 2 - 1) as f32),
            ),
            (
                Square::at(0, 0).of_size(2),
                Square::at(0, 0).of_size(1),
                Some(0.25),
            ),
            (
                Square::at(0, 0).of_size(1),
                Square::at(1, 1).of_size(1),
                None,
            ),
            (
                Square::at(0, 0).of_size(1),
                Square::at(1, 0).of_size(1),
                None,
            ),
            (
                Square::at(0, 0).of_size(1),
                Square::at(0, 1).of_size(1),
                None,
            ),
        ];

        for (r1, r2, test_iou) in tests.iter() {
            let calc_iou = intersection_over_union(*r1, *r2);
            assert!(
                calc_iou.is_none() && test_iou.is_none()
                    || calc_iou.is_some() && test_iou.is_some()
            );

            if let Some((calc, test)) = calc_iou.zip(*test_iou) {
                assert_abs_diff_eq!(calc, test);
            };
        }
    }
}
