use na::{
    Affine2, Matrix2, MatrixMN, Point2, Rotation2, SimilarityMatrix2, Translation2, Vector2,
    Vector3, U2, U3,
};

#[inline]
pub fn scale_and_translate_fast(point: &Point2<i8>, transform: &Vector3<i32>) -> Point2<u32> {
    let x = (((transform.x << 8) + (point.x as i32) * transform.z) >> 8) as u32;
    let y = (((transform.y << 8) + (point.y as i32) * transform.z) >> 8) as u32;
    Point2::new(x, y)
}

#[inline]
pub fn find_similarity(
    from_points: &[Point2<f32>],
    to_points: &[Point2<f32>],
) -> SimilarityMatrix2<f32> {
    // see paper:
    //
    // "Least-squares estimation of transformation parameters between two point patterns"
    // http://cis.jhu.edu/software/lddmm-similitude/umeyama.pdf
    //
    // Equations 34-43.
    assert_eq!(from_points.len(), to_points.len());
    let size_recip: f32 = (from_points.len() as f32).recip();

    let mean_from: Vector2<f32> = from_points
        .iter()
        .fold(Vector2::zeros(), |acc, p| acc + p.coords)
        .scale(size_recip);

    let mean_to: Vector2<f32> = to_points
        .iter()
        .fold(Vector2::zeros(), |acc, p| acc + p.coords)
        .scale(size_recip);

    let mut sigma_from = 0f32;
    let mut cov = Matrix2::zeros();

    for (from_point, to_point) in from_points.iter().zip(to_points.iter()) {
        let d_from = from_point.coords - mean_from;
        let d_to = to_point.coords - mean_to;
        sigma_from += d_from.norm_squared();
        cov += d_to * d_from.transpose();
    }
    sigma_from *= size_recip;
    cov.scale_mut(size_recip);

    let (svd, det) = (cov.svd(true, true), cov.determinant());
    let u = svd.u.unwrap();
    let v_t = svd.v_t.unwrap();
    let d = Matrix2::from_diagonal(&svd.singular_values);
    let mut s = Matrix2::identity();

    if det < 0.0 || (det == 0.0 && (u.determinant() * v_t.determinant()) < 0.0) {
        s[if d[(1, 1)] < d[(0, 0)] {
            (1, 1)
        } else {
            (0, 0)
        }] = -1.0;
    }

    let rotation = u * s * v_t;
    let scale = if sigma_from != 0.0 {
        sigma_from.recip() * (d * s).trace()
    } else {
        1.0
    };

    let translation = mean_to - scale * (rotation * mean_from);

    SimilarityMatrix2::from_parts(
        Translation2::from(translation),
        Rotation2::from_matrix(&rotation),
        scale,
    )
}

#[inline]
pub fn find_affine(
    from_points: &[Point2<f32>],
    to_points: &[Point2<f32>],
    eps: f32,
) -> Result<Affine2<f32>, &'static str> {
    assert!(from_points.len() >= 3);
    assert_eq!(from_points.len(), to_points.len());

    let input = MatrixMN::<f32, U3, U3>::from_iterator(
        from_points
            .iter()
            .take(3)
            .flat_map(|point| *point.to_homogeneous().data),
    );

    let transformed = MatrixMN::<f32, U2, U3>::from_iterator(
        to_points
            .iter()
            .take(3)
            .flat_map(|point| *point.coords.data),
    );

    let mut transform = (transformed * input.pseudo_inverse(eps)?).fixed_resize::<U3, U3>(0.0);
    transform[(2, 2)] = 1.0;
    Ok(Affine2::from_matrix_unchecked(transform))
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
        let from_points = vec![
            Point2::new(1.0, 1.0),
            Point2::new(-2.0, 0.0),
            Point2::new(2.0, -0.5),
            Point2::new(0.0, 0.0),
            Point2::new(-1.0, -1.0),
        ];

        let angles: Vec<f32> = vec![
            0f32,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::FRAC_PI_3,
            std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_6,
        ];
        let scales: Vec<f32> = vec![1.0, 3f32.recip(), 0.5, 1.5, 2.0];
        let xs: Vec<f32> = vec![-2.0, -1.0, 0.0, 1.0];
        let ys: Vec<f32> = vec![-2.0, -1.0, 0.0, 1.0];

        for scale in scales.iter() {
            for angle in angles.iter() {
                for x in xs.iter() {
                    for y in ys.iter() {
                        let test = SimilarityMatrix2::new(Vector2::new(*x, *y), *angle, *scale);
                        println!("> test");
                        println!("  translation: {}", test.isometry.translation.vector);
                        println!(
                            "  rotation:    {}",
                            test.isometry.rotation.angle().to_degrees()
                        );
                        println!("  scale:       {}", test.scaling());

                        let to_points: Vec<Point2<f32>> = from_points
                            .iter()
                            .map(|point| test.transform_point(point))
                            .collect();

                        let transform = find_similarity(&from_points, &to_points);
                        println!("> found");
                        println!("  translation: {}", transform.isometry.translation.vector);
                        println!(
                            "  rotation:    {}",
                            transform.isometry.rotation.angle().to_degrees()
                        );
                        println!("  scale:       {}", transform.scaling());
                        assert_abs_diff_eq!(transform, test, epsilon = 0.001);
                    }
                }
            }
        }
    }

    #[test]
    fn check_find_affine() {
        let test = Affine2::from_matrix_unchecked(MatrixMN::<f32, U3, U3>::new(
            3.07692308,
            8.46153846,
            -546.15384615,
            -1.15384615,
            -6.92307692,
            392.30769231,
            0.0,
            0.0,
            1.0,
        ));
        let from_points = vec![
            Point2::new(40.0, 50.0),
            Point2::new(100.0, 40.0),
            Point2::new(150.0, 10.0),
        ];

        let to_points: Vec<Point2<f32>> = from_points
            .iter()
            .map(|point| test.transform_point(point))
            .collect();

        let affine = find_affine(&from_points, &to_points, 0.0001).unwrap();
        assert_abs_diff_eq!(test, affine, epsilon = 0.001);
    }
}
