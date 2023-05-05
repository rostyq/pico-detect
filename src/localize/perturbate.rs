use rand::distributions::Uniform;
use rand::{Rng, RngCore};

use crate::geometry::Target;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perturbator {
    pub scale: Uniform<f32>,
    pub translate: Uniform<f32>,
}

impl Default for Perturbator {
    #[inline]
    fn default() -> Self {
        Self {
            scale: Uniform::new(0.925, 0.94),
            translate: Uniform::new(-0.075, 0.075),
        }
    }
}

impl Perturbator {
    #[inline]
    pub fn run<R, F>(&self, rng: &mut R, count: usize, init: Target, f: F)
    where
        R: RngCore,
        F: FnMut(Target),
    {
        perturbate(rng, self.scale, self.translate, count, init, f)
    }
}

#[inline]
pub fn perturbate<R, F>(
    rng: &mut R,
    scale: Uniform<f32>,
    translate: Uniform<f32>,
    count: usize,
    init: Target,
    mut f: F,
) where
    R: RngCore,
    F: FnMut(Target),
{
    let size = init.size();

    for _ in 0..count {
        let s = size * rng.sample(scale);

        let x = s.mul_add(rng.sample(translate), init.x());
        let y = s.mul_add(rng.sample(translate), init.y());

        f(Target::new(x, y, s));
    }
}
