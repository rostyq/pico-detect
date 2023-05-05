mod localizer;
pub mod perturbate;

use image::{GenericImageView, Luma};
pub use localizer::Localizer;

use nalgebra::Point2;
use perturbate::Perturbator;
use rand::RngCore;

use crate::Target;

#[derive(Debug, Clone, Copy)]
pub struct LocalizePerturbate {
    pub perturbator: Perturbator,
    pub runs: usize,
}

impl Default for LocalizePerturbate {
    #[inline]
    fn default() -> Self {
        Self {
            perturbator: Default::default(),
            runs: 15,
        }
    }
}

impl LocalizePerturbate {
    #[inline]
    pub fn new(runs: usize) -> Self {
        Self {
            perturbator: Default::default(),
            runs,
        }
    }

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
        I: GenericImageView<Pixel = Luma<u8>>,
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
