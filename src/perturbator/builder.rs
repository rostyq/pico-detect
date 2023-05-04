use rand::distributions::Uniform;

use super::Perturbator;

#[derive(Clone, Copy, Debug, Default)]
pub struct PerturbatorBuilder {
    pub srange: Option<(f32, f32)>,
    pub trange: Option<(f32, f32)>,
}

impl PerturbatorBuilder {
    #[inline]
    pub fn with_scale_range(mut self, low: f32, high: f32) -> Self {
        self.srange = Some((low, high));
        self
    }

    #[inline]
    pub fn with_translation_range(mut self, low: f32, high: f32) -> Self {
        self.trange = Some((low, high));
        self
    }

    #[inline]
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
        })
    }
}
