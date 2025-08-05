mod localizer;
pub mod perturbate;

use image::Luma;
pub use localizer::Localizer;

use nalgebra::Point2;
use perturbate::Perturbator;
use pixelutil_image::ExtendedImageView;
use rand::RngCore;

use crate::Target;

/// Implements object localization with perturbation.
#[derive(Debug, Clone, Copy)]
pub struct LocalizePerturbate {
    /// Perturbator to apply to the target.
    pub perturbator: Perturbator,
    /// Number of perturbations to run.
    pub runs: usize,
}

impl Default for LocalizePerturbate {
    /// Creates with default perturbator and runs count set to 15.
    #[inline]
    fn default() -> Self {
        Self {
            perturbator: Default::default(),
            runs: 15,
        }
    }
}

impl LocalizePerturbate {
    /// Creates a new instance with the specified number of runs
    /// with a default perturbator.
    #[inline]
    pub fn new(runs: usize) -> Self {
        Self {
            perturbator: Default::default(),
            runs,
        }
    }

    /// Applies perturbations to the target and runs the localizer on each perturbed target.
    #[inline]
    pub fn run<R, I>(
        &self,
        localizer: &Localizer,
        rng: &mut R,
        image: &I,
        target: Target,
    ) -> Point2<f32>
    where
        R: RngCore,
        I: ExtendedImageView<Pixel = Luma<u8>>,
    {
        let mut xs: Vec<f32> = Vec::with_capacity(self.runs);
        let mut ys: Vec<f32> = Vec::with_capacity(self.runs);

        self.perturbator.run(rng, self.runs, target, |t| {
            let p = localizer.localize(image, t);

            xs.push(p.x);
            ys.push(p.y);
        });

        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (self.runs - 1) / 2;

        Point2::new(xs[index], ys[index])
    }
}
