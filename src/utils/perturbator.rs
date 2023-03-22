use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

use super::target::Target;

#[derive(Clone, Debug)]
pub struct Perturbator {
    pub sdist: Uniform<f32>,
    pub tdist: Uniform<f32>,
    pub rng: XorShiftRng,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PerturbatorBuilder {
    pub seed: Option<u64>,
    pub srange: Option<(f32, f32)>,
    pub trange: Option<(f32, f32)>,
}

impl PerturbatorBuilder {
    pub fn with_seed(mut self, value: u64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn with_scale_range(mut self, low: f32, high: f32) -> Self {
        self.srange = Some((low, high));
        self
    }

    pub fn with_translation_range(mut self, low: f32, high: f32) -> Self {
        self.trange = Some((low, high));
        self
    }

    pub fn build(self) -> Result<Perturbator, &'static str> {
        let (slow, shigh) = self.srange.unwrap_or((0.925, 0.94));
        let (tlow, thigh) = self.trange.unwrap_or((-0.075, 0.075));

        if slow > shigh || slow < 0.9 || shigh > 1.0 {
            return Err("invalid scale perturb range");
        }

        if tlow > thigh || tlow < -0.1 || thigh > 0.1 {
            return Err("invalid translation perturb range");
        }

        Ok(Perturbator {
            sdist: Uniform::new(slow, shigh),
            tdist: Uniform::new(tlow, thigh),
            rng: XorShiftRng::seed_from_u64(self.seed.unwrap_or(42)),
        })
    }
}

impl Perturbator {
    pub fn builder() -> PerturbatorBuilder {
        Default::default()
    }

    #[inline]
    pub fn run<F>(&mut self, n: usize, init: Target, mut f: F)
    where
        F: FnMut(Target),
    {
        let size = init.size();

        for _ in 0..n {
            let s = size * self.rng.sample(self.sdist);

            let x = s.mul_add(self.rng.sample(self.tdist), init.x());
            let y = s.mul_add(self.rng.sample(self.tdist), init.y());

            f(Target::new(x, y, s));
        }
    }
}