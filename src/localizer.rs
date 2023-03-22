use std::cmp::Ordering;
use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};
use nalgebra::{Point2, Translation2, Vector2};

use crate::nodes::ComparisonNode;
use crate::utils::perturbator::{Perturbator, PerturbatorBuilder};
use crate::utils::target::Target;

type Tree = Vec<ComparisonNode>;
type Predictions = Vec<Vector2<f32>>;
type Stage = Vec<(Tree, Predictions)>;

/// Implements object localization using decision trees.
///
/// Details available [here](https://tehnokv.com/posts/puploc-with-trees/).
pub struct Localizer {
    depth: usize,
    dsize: usize,
    scale: f32,
    stages: Vec<Stage>,
}

impl Localizer {
    /// Estimate object location on the image
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// TODO
    pub fn localize<I>(&self, image: &I, roi: Target) -> Point2<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let Target {
            mut point,
            mut size,
        } = roi;

        for stage in self.stages.iter() {
            let mut translation = Translation2::identity();
            let p = Point2::new(point.x as i64, point.y as i64);
            let s = size as u32;

            for (codes, preds) in stage.iter() {
                let idx = (0..self.depth).fold(0, |idx, _| {
                    2 * idx + 1 + codes[idx].bintest(image, p, s) as usize
                });
                let lutidx = (idx + 1) - self.dsize;

                translation.vector += preds[lutidx];
            }

            translation.vector.scale_mut(size);
            *point = *translation.transform_point(&point);
            size *= self.scale;
        }

        point
    }

    /// Load localizer from a readable source.
    pub fn load(mut readable: impl Read) -> Result<Self, Error> {
        let mut buffer: [u8; 4] = [0u8; 4];
        readable.read_exact(&mut buffer)?;
        let nstages = i32::from_le_bytes(buffer) as usize;

        readable.read_exact(&mut buffer)?;
        let scale = f32::from_le_bytes(buffer);

        readable.read_exact(&mut buffer)?;
        let ntrees = i32::from_le_bytes(buffer) as usize;

        readable.read_exact(&mut buffer)?;
        let depth = i32::from_le_bytes(buffer) as usize;
        let pred_size: usize = match 2usize.checked_pow(depth as u32) {
            Some(value) => value,
            None => return Err(Error::new(ErrorKind::Other, "depth overflow")),
        };
        let code_size = pred_size - 1;

        let mut stages = Vec::with_capacity(nstages);

        for _ in 0..nstages {
            let mut stage: Stage = Vec::with_capacity(ntrees);

            for _ in 0..ntrees {
                let mut tree: Tree = Vec::with_capacity(code_size);
                let mut predictions: Predictions = Vec::with_capacity(pred_size);

                for _ in 0..code_size {
                    readable.read_exact(&mut buffer)?;
                    let node = ComparisonNode::from(buffer);
                    tree.push(node);
                }

                for _ in 0..pred_size {
                    readable.read_exact(&mut buffer)?;
                    let y = f32::from_le_bytes(buffer);

                    readable.read_exact(&mut buffer)?;
                    let x = f32::from_le_bytes(buffer);

                    predictions.push(Vector2::new(x, y));
                }

                stage.push((tree, predictions));
            }

            stages.push(stage);
        }

        Ok(Self {
            depth,
            dsize: pred_size,
            scale,
            stages,
        })
    }
}

pub struct PerturbatingLocalizer {
    pub model: Localizer,
    pub perturbator: Perturbator,
    pub perturbs: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PerturbatingLocalizerBuilder {
    pub perturbator_builder: PerturbatorBuilder,
    pub perturbs: Option<usize>,
}

impl PerturbatingLocalizerBuilder {
    pub fn with_perturbs(mut self, value: usize) -> Self {
        self.perturbs = Some(value);
        self
    }

    pub fn map_perturbator_builder<F: FnOnce(PerturbatorBuilder) -> PerturbatorBuilder>(
        mut self,
        f: F,
    ) -> Self {
        self.perturbator_builder = f(self.perturbator_builder);
        self
    }

    pub fn build(self, model: Localizer) -> Result<PerturbatingLocalizer, &'static str> {
        if let Some(value) = self.perturbs {
            if (value % 2) == 0 {
                return Err("`nperturbs` should be odd");
            }
        }

        Ok(PerturbatingLocalizer {
            perturbs: self.perturbs.unwrap_or(15),
            perturbator: self.perturbator_builder.build()?,
            model,
        })
    }
}

impl PerturbatingLocalizer {
    pub fn builder() -> PerturbatingLocalizerBuilder {
        Default::default()
    }

    pub fn localize<I>(&mut self, image: &I, roi: Target) -> Point2<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut xs: Vec<f32> = Vec::with_capacity(self.perturbs);
        let mut ys: Vec<f32> = Vec::with_capacity(self.perturbs);

        let model = &self.model;

        self.perturbator.run(self.perturbs, roi, |s| {
            let p = model.localize(image, s);
            xs.push(p.x);
            ys.push(p.y);
        });

        #[inline]
        fn compare(a: &f32, b: &f32) -> Ordering {
            a.partial_cmp(b).unwrap()
        }
        xs.sort_by(compare);
        ys.sort_by(compare);

        let index = (self.perturbs - 1) / 2;

        Point2::new(xs[index], ys[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pupil_localizer_model_loading() {
        let puploc = Localizer::load(
            include_bytes!("../models/pupil.localizer.bin")
                .to_vec()
                .as_slice(),
        )
        .expect("parsing failed");
        let stages = &puploc.stages;
        let trees = stages[0].len();

        assert_eq!(5, stages.len());
        assert_eq!(20, trees);
        assert_eq!(10, puploc.depth);
        assert_eq!(80, (puploc.scale * 100.0) as u32);

        let dsize = 2usize.pow(puploc.depth as u32);

        let first_node = ComparisonNode::from([30i8, -16i8, 125i8, 14i8]);
        let last_node = ComparisonNode::from([-125i8, 26i8, 15i8, 98i8]);
        assert_eq!(first_node, stages[0][0].0[0]);
        assert_eq!(
            last_node,
            stages[stages.len() - 1][trees - 1].0[dsize - 1 - 1]
        );

        let first_pred_test = Vector2::new(-0.08540829f32, 0.04436668f32);
        let last_pred_test = Vector2::new(0.05820565f32, 0.02249731f32);
        let first_pred = stages[0][0].1[0];
        let last_pred = stages[stages.len() - 1][trees - 1].1[dsize - 1];
        assert_abs_diff_eq!(first_pred_test, first_pred);
        assert_abs_diff_eq!(last_pred_test, last_pred);
    }
}
