use std::{
    convert::{TryFrom, TryInto},
    ops::Range,
};

use rand::{
    distr::{uniform::Error, Uniform},
    Rng, RngCore,
};

use crate::geometry::Target;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Perturbator {
    pub scale: Uniform<f32>,
    pub translate: Uniform<f32>,
}

impl Perturbator {
    /// Creates a new perturbator with the specified scale and translation ranges.
    #[inline]
    pub fn from_ranges(scale: Range<f32>, translate: Range<f32>) -> Result<Self, Error> {
        Ok(Self {
            scale: scale.try_into()?,
            translate: translate.try_into()?,
        })
    }
}

impl Default for Perturbator {
    #[inline]
    fn default() -> Self {
        Self {
            scale: Uniform::try_from(0.925..0.94).unwrap(),
            translate: Uniform::try_from(-0.075..0.075).unwrap(),
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
