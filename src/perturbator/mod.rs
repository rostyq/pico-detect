mod builder;

use rand::distributions::Uniform;
use rand::{Rng, RngCore};

use crate::utils::target::Target;

pub use builder::PerturbatorBuilder;

#[derive(Clone, Debug)]
pub struct Perturbator {
    pub sdist: Uniform<f32>,
    pub tdist: Uniform<f32>,
}

impl Perturbator {
    #[inline]
    pub fn builder() -> PerturbatorBuilder {
        Default::default()
    }

    #[inline]
    pub fn run<R, F>(&self, rng: &mut R, n: usize, init: Target, mut f: F)
    where
        R: RngCore,
        F: FnMut(Target),
    {
        let size = init.size();

        for _ in 0..n {
            let s = size * rng.sample(self.sdist);

            let x = s.mul_add(rng.sample(self.tdist), init.x());
            let y = s.mul_add(rng.sample(self.tdist), init.y());

            f(Target::new(x, y, s));
        }
    }
}