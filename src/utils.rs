use image::GenericImageView;

#[inline]
fn saturate_coordinate(value: u32, bound: u32) -> u32 {
    if value < bound {
        value
    } else {
        bound - 1
    }
}

#[inline]
pub fn saturating_get_pixel<I>(image: &I, x: i32, y: i32) -> I::Pixel
where
    I: GenericImageView,
{
    let x = if x.is_negative() {
        0u32
    } else {
        saturate_coordinate(x as u32, image.width())
    };
    let y = if y.is_negative() {
        0u32
    } else {
        saturate_coordinate(y as u32, image.height())
    };
    unsafe { image.unsafe_get_pixel(x, y) }
}

#[inline]
pub fn get_pixel_with_fallback<I>(image: &I, x: i32, y: i32, fallback: I::Pixel) -> I::Pixel
where
    I: GenericImageView,
{
    if x.is_negative() || y.is_negative() {
        fallback
    } else {
        let x = x as u32;
        let y = y as u32;
        if image.in_bounds(x, y) {
            unsafe { image.unsafe_get_pixel(x, y) }
        } else {
            fallback
        }
    }
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
            assert_eq!(saturating_get_pixel(&image, x, y), lum);

            let fallback = Luma::from([0u8]);
            assert_eq!(
                get_pixel_with_fallback(&image, x, y, fallback.clone()),
                if *should_fallback { fallback } else { lum }
            );
        }
    }
}
