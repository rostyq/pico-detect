use derive_builder::Builder;
use imageproc::rect::Rect;

use crate::geometry::Square;

#[derive(Copy, Clone, Debug, Builder, PartialEq)]
#[builder(build_fn(validate = "Multiscaler::validate"))]
pub struct Multiscaler {
    min_size: u32,
    max_size: u32,
    #[builder(default = "0.1")]
    shift_factor: f32,
    #[builder(default = "1.1")]
    scale_factor: f32,
}

impl Multiscaler {
    #[inline]
    pub fn validate(builder: &MultiscalerBuilder) -> Result<(), String> {
        if let Some(value) = builder.min_size {
            if value == 0 {
                return Err("`min_size` should be non zero".into());
            }
        }

        if let Some((min_size, max_size)) = builder.min_size.zip(builder.max_size) {
            if min_size > max_size {
                return Err("`max_size` should be greater than `min_size`".into());
            }
        }

        if let Some(value) = builder.shift_factor {
            if !(0.0..=1.0).contains(&value) {
                return Err("`shift_factor` should be in `(0, 1]` range".into());
            }
        }

        if let Some(value) = builder.scale_factor {
            if value < 1.0 {
                return Err("`scale_factor` should be greater than `1.0`".into());
            }
        };

        Ok(())
    }

    pub fn builder() -> MultiscalerBuilder {
        Default::default()
    }

    pub fn min_size(&self) -> u32 {
        self.min_size
    }

    pub fn max_size(&self) -> u32 {
        self.max_size
    }

    pub fn shift_factor(&self) -> f32 {
        self.shift_factor
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[inline]
    pub fn run<F>(&self, rect: Rect, f: F)
    where
        F: FnMut(Square),
    {
        multiscale(
            self.min_size,
            self.max_size,
            self.shift_factor,
            self.scale_factor,
            rect,
            f,
        )
    }

    #[inline]
    pub fn count(&self, rect: Rect) -> usize {
        let mut count = 0;
        self.run(rect, |_| count += 1);
        count
    }

    #[inline]
    pub fn collect(&self, rect: Rect) -> Vec<Square> {
        let mut result = Vec::with_capacity(self.count(rect));
        self.run(rect, |s| result.push(s));
        result
    }
}

#[inline]
pub fn multiscale<F>(
    min_size: u32,
    max_size: u32,
    shift_factor: f32,
    scale_factor: f32,
    rect: Rect,
    mut f: F,
) where
    F: FnMut(Square),
{
    let mut size = min_size;

    let start_x = rect.left();
    let start_y = rect.top();

    let right = start_x + rect.width() as i32;
    let bottom = start_y + rect.height() as i32;

    while size <= max_size {
        let sizef = size as f32;
        let step: usize = 1.max((sizef * shift_factor) as usize);

        let end_x = right - size as i32;
        let end_y = bottom - size as i32;

        for y in (start_y..=end_y).step_by(step) {
            for x in (start_x..=end_x).step_by(step) {
                f(Square::new(x as i64, y as i64, size))
            }
        }
        size = (sizef * scale_factor) as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiscale_run() {
        let ms = Multiscaler::builder()
            .min_size(1)
            .max_size(4)
            .scale_factor(2.0)
            .shift_factor(1.0)
            .build()
            .unwrap();

        ms.run(Rect::at(0, 0).of_size(4, 4), |s| println!("{:?}", s));
    }
}
