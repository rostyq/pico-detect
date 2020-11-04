use std::cmp;

use image::{GenericImageView, Luma};
use nalgebra::Point2;

use super::detection::Detection;
use super::detector::Detector;
use super::geometry::ISimilarity2;

/// Multiscale detection for `Detector`.
#[derive(Copy, Clone, Debug)]
pub struct MultiScale {
    min_size: u32,
    max_size: u32,
    shift_factor: f32,
    scale_factor: f32,
    pad_x: (i32, i32),
    pad_y: (i32, i32),
}

impl Default for MultiScale {
    fn default() -> Self {
        Self {
            min_size: 100,
            max_size: 1000,
            shift_factor: 0.1,
            scale_factor: 1.1,
            pad_x: (0, 0),
            pad_y: (0, 0),
        }
    }
}

impl MultiScale {
    /// MultiScale with padded sliding window area.
    pub fn with_padding(mut self, x: (i32, i32), y: (i32, i32)) -> Self {
        self.pad_x = x;
        self.pad_y = y;
        self
    }

    /// MultiScale with min/max sliding window size.
    pub fn with_size_range(mut self, min_size: u32, max_size: u32) -> Self {
        assert!(
            max_size > min_size,
            "Max size should be greater than min size"
        );
        self.min_size = min_size;
        self.max_size = max_size;
        self
    }

    /// MultiScale with shift factor.
    ///
    /// Means how far to move a sliding window by fraction of its size: (0..1).
    pub fn with_shift_factor(mut self, factor: f32) -> Self {
        assert!(factor > 0.0, "Shift factor should be positive");
        assert!(factor < 1.0, "Shift factor should less 1.0");
        self.shift_factor = factor;
        self
    }

    /// MultiScale with scale factor.
    ///
    /// For multiscale processing: resize the detection window by fraction
    /// of its size when moving to the higher scale. Must be greater that 1.
    pub fn with_scale_factor(mut self, factor: f32) -> Self {
        assert!(factor > 1.0, "Shift factor should be greater than 1");
        self.scale_factor = factor;
        self
    }

    #[inline]
    fn sliding_window_bbox(
        &self,
        width: u32,
        height: u32,
        offset: i32,
    ) -> (Point2<i32>, Point2<i32>) {
        let start_x = self.pad_x.0 + offset;
        let end_x = (width as i32) - self.pad_x.1 - offset;
        let start_y = self.pad_y.0 + offset;
        let end_y = (height as i32) - self.pad_y.1 - offset;
        (Point2::new(start_x, start_y), Point2::new(end_x, end_y))
    }

    /// Run multiscale cascade detector
    /// and push detections to the existing collection.
    ///
    /// ### Returns
    ///
    /// New detections count.
    #[inline]
    pub fn run_mut<I>(&self, detector: &Detector, image: &I, detections: &mut Vec<Detection>) -> usize
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let (width, height) = (image.width(), image.height());
        let mut size = self.min_size;
        let mut new_detections = 0;

        while size <= self.max_size {
            let sizef = size as f32;
            let step = cmp::max((sizef * self.shift_factor) as usize, 1);
            let offset = (size / 2 + 1) as i32;
            let (start, end) = self.sliding_window_bbox(width, height, offset);

            for y in (start.y..end.y).step_by(step) {
                for x in (start.x..end.x).step_by(step) {
                    if let Some(score) =
                        detector.classify(image, ISimilarity2::from_components(x, y, size))
                    {
                        detections
                            .push(Detection::from_components(x as f32, y as f32, sizef, score));
                        new_detections += 1;
                    }
                }
            }
            size = (sizef * self.scale_factor) as u32;
        }
        new_detections
    }

    /// Same as `run_mut` but creates new empty collection for detections.
    pub fn run<I>(&self, detector: &Detector, image: &I) -> Vec<Detection>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut detections = Vec::new();
        self.run_mut(detector, image, &mut detections);
        detections
    }
}
