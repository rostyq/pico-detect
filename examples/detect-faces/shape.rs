use pico_detect::utils::{Point2, Square};

pub enum Shape5 {
    LeftOuterEyeCorner = 0,
    LeftInnerEyeCorner = 1,
    RightOuterEyeCorner = 2,
    RightInnerEyeCorner = 3,
    #[allow(dead_code)]
    Nose = 4,
}

impl Shape5 {
    pub fn size() -> usize {
        5
    }

    #[allow(dead_code)]
    pub fn find_eye_centers(shape: &[Point2<f32>]) -> (Point2<f32>, Point2<f32>) {
        assert_eq!(shape.len(), Self::size());
        (
            nalgebra::center(
                &shape[Self::LeftInnerEyeCorner as usize],
                &shape[Self::LeftOuterEyeCorner as usize],
            ),
            nalgebra::center(
                &shape[Self::RightInnerEyeCorner as usize],
                &shape[Self::RightOuterEyeCorner as usize],
            ),
        )
    }

    pub fn find_eyes_roi(shape: &[Point2<f32>]) -> (Square, Square) {
        assert_eq!(shape.len(), Self::size());
        let (li, lo) = (
            &shape[Self::LeftInnerEyeCorner as usize],
            &shape[Self::LeftOuterEyeCorner as usize],
        );
        let (ri, ro) = (
            &shape[Self::RightInnerEyeCorner as usize],
            &shape[Self::RightOuterEyeCorner as usize],
        );

        let (dl, dr) = (lo - li, ri - ro);
        let (l, r) = (li + dl.scale(0.5), ro + dr.scale(0.5));

        let ls = dl.norm() * 1.1;
        let rs = dr.norm() * 1.1;

        let lh = ls / 2.0;
        let rh = rs / 2.0;

        (
            Square::new((l.x - lh) as i64, (l.y - lh) as i64, ls as u32),
            Square::new((r.x - rh) as i64, (r.y - rh) as i64, rs as u32),
        )
    }
}