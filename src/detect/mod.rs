mod detection;
mod detector;
mod padding;

pub mod clusterize;
pub mod multiscale;

use image::{GenericImageView, Luma};
use derive_builder::Builder;

use crate::geometry::Target;

use clusterize::Clusterizer;
use multiscale::Multiscaler;

pub use detection::Detection;
pub use detector::Detector;
pub use padding::Padding;

#[derive(Debug, Clone, Copy, Builder)]
#[builder]
pub struct DetectMultiscale {
    pub multiscaler: Multiscaler,
    #[builder(default)]
    pub clusterizer: Clusterizer,
    #[builder(default)]
    pub padding: Padding,
}

impl DetectMultiscale {
    #[inline]
    pub fn builder() -> DetectMultiscaleBuilder {
        Default::default()
    }

    #[inline]
    pub fn run<I>(&self, detector: &Detector, image: &I) -> Vec<Detection<Target>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut detections = Vec::new();
        
        self.multiscaler.run(self.padding.image_rect(image), |region| {
            if let Some(detection) = detector.detect(image, region) {
                detections.push(detection);
            }
        });

        let mut clusters = Vec::new();

        self.clusterizer.clusterize(&mut detections, &mut clusters);

        clusters
    }
}
