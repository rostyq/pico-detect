use image::{GenericImageView, Luma};
use nalgebra::Point2;

#[inline]
pub fn get_nearest_pixel_i64<I>(image: &I, x: i64, y: i64) -> I::Pixel
where
    I: GenericImageView,
{
    let (x0, y0, w, h) = image_bounds_as_i64(image);

    let x1 = x0 + w - 1;
    let y1 = y0 + h - 1;

    #[allow(clippy::manual_clamp)]
    unsafe { image.unsafe_get_pixel(x.max(x0).min(x1) as u32, y.max(y0).min(y1) as u32) }
}

#[inline]
pub fn get_nearest_luma_by_point<I, T: Copy>(image: &I, point: Point2<i64>) -> T
where
    I: GenericImageView<Pixel = Luma<T>>,
{
    get_nearest_pixel_i64(image, point.x, point.y).0[0]
}

#[inline]
pub fn get_luma_by_point_f32<I, T: Copy>(image: &I, point: Point2<f32>) -> Option<T>
where
    I: GenericImageView<Pixel = Luma<T>>,
{
    get_pixel_i64(image, point.x as i64, point.y as i64).map(|p| p.0[0])
}

#[inline]
pub fn image_bounds_as_i64<I>(image: &I) -> (i64, i64, i64, i64)
where
    I: GenericImageView,
{
    let (ix, iy, iw, ih) = image.bounds();
    (ix as i64, iy as i64, iw as i64, ih as i64)
}

#[inline]
pub fn in_bounds_i64<I>(image: &I, x: i64, y: i64) -> bool
where
    I: GenericImageView,
{
    let (ix, iy, iw, ih) = image_bounds_as_i64(image);
    x >= ix && x < ix + iw && y >= iy && y < iy + ih
}

#[inline]
pub fn get_pixel_i64<I>(image: &I, x: i64, y: i64) -> Option<I::Pixel>
where
    I: GenericImageView,
{
    in_bounds_i64(image, x, y).then(|| unsafe { image.unsafe_get_pixel(x as u32, y as u32) })
}
