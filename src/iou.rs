use nalgebra::Point2;

use super::utils::roi_to_bbox;

/// Intersection over Union (IoU)
#[inline]
pub fn calculate_iou(p0: Point2<f32>, p1: Point2<f32>, s0: f32, s1: f32) -> f32 {
    let b0 = roi_to_bbox(p0, s0);
    let b1 = roi_to_bbox(p1, s1);

    let ix = 0f32.max(b0.1.x.min(b1.1.x) - b0.0.x.max(b1.0.x));
    let iy = 0f32.max(b0.1.y.min(b1.1.y) - b0.0.y.max(b1.0.y));

    let inter_square = ix * iy;
    let union_square = (s0 * s0 + s1 * s1) - inter_square;

    inter_square / union_square
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_iou() {
        let tests: Vec<((f32, f32, f32), (f32, f32, f32), f32)> = vec![
            (
                (100.0, 100.0, 50.0),
                (200.0, 100.0, 50.0),
                0.0,
            ),
            (
                (100.0, 100.0, 50.0),
                (100.0, 200.0, 50.0),
                0.0,
            ),
            (
                (100.0, 100.0, 50.0),
                (200.0, 200.0, 50.0),
                0.0,
            ),
            (
                (100.0, 100.0, 50.0),
                (100.0, 100.0, 50.0),
                1.0,
            ),
            (
                (100.0, 100.0, 50.0),
                (125.0, 100.0, 50.0),
                0.3333333,
            ),
            (
                (100.0, 100.0, 50.0),
                (100.0, 125.0, 50.0),
                0.3333333,
            ),
            (
                (100.0, 100.0, 60.0),
                (125.0, 125.0, 65.0),
                0.21908471,
            ),
        ];

        for ((x0, y0, s0), (x1, y1, s1), iou) in tests.iter() {
            assert_abs_diff_eq!(
                calculate_iou(Point2::new(*x0, *y0), Point2::new(*x1, *y1), *s0, *s1),
                iou
            );
        }
    }
}
