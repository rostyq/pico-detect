use image::GenericImageView;
use nalgebra::Point2;

#[inline(always)]
pub fn get_nearest_pixel_i64<I>(image: &I, x: i64, y: i64) -> I::Pixel
where
    I: GenericImageView,
{
    let (ix, iy, iw, ih) = image.bounds();

    let x0 = ix as i64;
    let x1 = (ix + iw - 1) as i64;

    let y0 = iy as i64;
    let y1 = (iy + ih - 1) as i64;

    unsafe { image.unsafe_get_pixel(x.clamp(x0, x1) as u32, y.clamp(y0, y1) as u32) }
}

#[inline(always)]
pub fn in_bounds_i64<I>(image: &I, x: i64, y: i64) -> bool
where
    I: GenericImageView,
{
    let (ix, iy, iw, ih) = image.bounds();
    let (ix, iy, iw, ih) = (ix as i64, iy as i64, iw as i64, ih as i64);
    x >= ix && x < ix + iw && y >= iy && y < iy + ih
}

#[inline(always)]
pub fn get_pixel_i64<I>(image: &I, x: i64, y: i64) -> Option<I::Pixel>
where
    I: GenericImageView,
{
    in_bounds_i64(image, x, y).then(|| unsafe { image.unsafe_get_pixel(x as u32, y as u32) })
}

/// (x, y, size) -> (x0, x1, y0, y1)
#[inline]
pub fn roi_to_bbox(point: Point2<f32>, size: f32) -> (Point2<f32>, Point2<f32>) {
    let h = size / 2.0;
    (
        Point2::new(point.x - h, point.y - h),
        Point2::new(point.x + h, point.y + h),
    )
}

pub fn odd_median_mut(numbers: &mut [f32]) -> f32 {
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    numbers[numbers.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    #[test]
    fn check_get_pixel() {
        let (width, height) = (64u32, 48u32);
        let mut image = GrayImage::new(width, height);
        image.put_pixel(0, 0, Luma::from([42u8]));
        image.put_pixel(width - 1, height - 1, Luma::from([255u8]));

        let test_coords = vec![
            (0f32, 0f32),
            (-10f32, -10f32),
            ((width as f32 - 1f32), (height as f32 - 1f32)),
            (width as f32, height as f32),
        ];

        let lum_values = vec![42u8, 42u8, 255u8, 255u8];

        let fallbacks = vec![false, true, false, true];

        for ((x, y), (lum_value, should_fallback)) in test_coords
            .iter()
            .zip(lum_values.iter().zip(fallbacks.iter()))
        {
            let x = *x as i32;
            let y = *y as i32;
            println!("x: {}, y: {}", x, y);
            let lum = Luma::from([*lum_value]);
            assert_eq!(get_nearest_pixel_i64(&image, x.into(), y.into()), lum);

            let fallback = Luma::from([0u8]);
            assert_eq!(
                get_pixel_i64(&image, x.into(), y.into()).unwrap_or(fallback.clone()),
                if *should_fallback { fallback } else { lum }
            );
        }
    }
}
