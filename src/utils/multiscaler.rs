use crate::utils::square::Square;
use crate::utils::padding::{Padding, PaddingBuilder};

#[derive(Copy, Clone, Debug)]
pub struct Multiscaler {
    min_size: u32,
    max_size: u32,
    shift_factor: f32,
    scale_factor: f32,
    padding: Padding,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MultiscalerBuilder {
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
    pub shift_factor: Option<f32>,
    pub scale_factor: Option<f32>,
    pub padding: PaddingBuilder,
}

impl MultiscalerBuilder {
    pub fn build(self) -> Result<Multiscaler, &'static str> {
        let min_size = self.min_size.ok_or("min_size is not set")?;
        let max_size = self.max_size.ok_or("max_size is not set")?;

        if max_size < min_size {
            return Err("`max_size` should be greater than `min_size`");
        }

        if min_size == 0 {
            return Err("`min_size` should be non zero");
        }

        let shift_factor = match self.shift_factor {
            Some(value) => {
                if value < 0.0 {
                    return Err("`shift_factor` should be positive");
                }
                if value > 1.0 {
                    return Err("`shift_factor` should be less than 1.0");
                }
                value
            }
            None => 0.1,
        };

        let scale_factor = match self.scale_factor {
            Some(value) => {
                if value < 1.0 {
                    return Err("`shift_factor` should be less than 1.0");
                }
                value
            }
            None => 1.1,
        };

        Ok(Multiscaler {
            min_size,
            max_size,
            shift_factor,
            scale_factor,
            padding: self.padding.build(),
        })
    }

    pub fn with_min_size(mut self, value: u32) -> Self {
        self.min_size = Some(value);
        self
    }

    pub fn with_max_size(mut self, value: u32) -> Self {
        self.max_size = Some(value);
        self
    }

    /// MultiScale with padded sliding window area.
    pub fn with_padding(mut self, padding: PaddingBuilder) -> Self {
        self.padding = padding;
        self
    }

    pub fn map_padding<F: FnOnce(PaddingBuilder) -> PaddingBuilder>(self, f: F) -> Self {
        self.with_padding(f(self.padding))
    }

    /// MultiScale with shift factor.
    ///
    /// Means how far to move a sliding window by fraction of its size: (0..1).
    pub fn with_shift_factor(mut self, value: f32) -> Self {
        self.shift_factor = Some(value);
        self
    }

    /// MultiScale with scale factor.
    ///
    /// For multiscale processing: resize the detection window by fraction
    /// of its size when moving to the higher scale. Must be greater that 1.0.
    pub fn with_scale_factor(mut self, value: f32) -> Self {
        self.scale_factor = Some(value);
        self
    }
}

impl Multiscaler {
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

    pub fn padding(&self) -> &Padding {
        &self.padding
    }

    #[inline]
    pub fn run<F>(&self, width: u32, height: u32, mut f: F)
    where
        F: FnMut(Square),
    {
        let mut size = self.min_size;
        let rect = self.padding.rect(width, height);

        let start_x = rect.left();
        let start_y = rect.top();

        let right = start_x + rect.width() as i32;
        let bottom = start_y + rect.height() as i32;

        while size <= self.max_size {
            let sizef = size as f32;
            let step: usize = 1.max((sizef * self.shift_factor) as usize);

            let end_x = right - size as i32;
            let end_y = bottom - size as i32;

            for y in (start_y..=end_y).step_by(step) {
                for x in (start_x..=end_x).step_by(step) {
                    f(Square::new(x as i64, y as i64, size))
                }
            }
            size = (sizef * self.scale_factor) as u32;
        }
    }

    #[inline]
    pub fn count(&self, width: u32, height: u32) -> usize {
        let mut count = 0;
        self.run(width, height, |_| count += 1);
        count
    }

    #[inline]
    pub fn collect(&self, width: u32, height: u32) -> Vec<Square> {
        let mut result = Vec::with_capacity(self.count(width, height));
        self.run(width, height, |rect| result.push(rect));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiscale_run() {
        let ms = Multiscaler::builder()
            .with_min_size(1)
            .with_max_size(4)
            .with_scale_factor(2.0)
            .with_shift_factor(1.0)
            .build()
            .unwrap();

        ms.run(4, 4, |s| println!("{:?}", s));
    }
}
