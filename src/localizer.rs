use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};
use nalgebra::{Point2, Similarity2, Translation2, Vector2};

use rand::distributions::Uniform;
use rand::{Rng, RngCore};

use super::bintest::ImageBintest;
use super::geometry::ISimilarity2;
use super::node::ComparisonNode;
use super::utils::odd_median_mut;

type Tree = Vec<ComparisonNode>;
type Predictions = Vec<Vector2<f32>>;
type Stage = Vec<(Tree, Predictions)>;
type Stages = Vec<Stage>;

/// Implements object localization using decision trees.
///
/// Details available [here](https://tehnokv.com/posts/puploc-with-trees/).
pub struct Localizer {
    depth: usize,
    dsize: usize,
    scale: f32,
    stages: Stages,
    distrs: (Uniform<f32>, Uniform<f32>),
}

impl Localizer {
    /// Estimate object location on the image
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// * `roi` -- similarity transform as region of interest:
    ///   - `roi.isometry.translation` region center position on image,
    ///   - TODO `roi.isometry.rotation` region rotation (have no effect),
    ///   - `roi.scaling` region size.
    pub fn localize<I>(&self, image: &I, mut roi: Similarity2<f32>) -> Point2<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let scaling = roi.scaling();
        for stage in self.stages.iter() {
            let mut translation = Translation2::identity();
            let roi_i32 = ISimilarity2::from(roi);

            for (codes, preds) in stage.iter() {
                let idx = (0..self.depth).fold(0, |idx, _| {
                    2 * idx + 1 + codes[idx].bintest(image, &roi_i32) as usize
                });
                let lutidx = (idx + 1) - self.dsize;

                translation.vector += preds[lutidx];
            }

            translation.vector.scale_mut(scaling);
            roi.append_translation_mut(&translation);

            roi.prepend_scaling_mut(self.scale);
        }

        Point2::from(roi.isometry.translation.vector)
    }

    /// Estimate object location on the image with perturbation to increase accuracy.
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// * `roi` -- similarity transform as region of interest:
    ///   - `roi.isometry.translation` region center position on image,
    ///   - `roi.isometry.rotation` region rotation (have no effect),
    ///   - `roi.scaling` region size.
    /// * `rng` - Source for randomness.
    /// * `nperturbs` - How many perturbations to make.
    pub fn perturb_localize<I>(
        &self,
        image: &I,
        initial_roi: Similarity2<f32>,
        mut rng: impl RngCore,
        nperturbs: usize,
    ) -> Point2<f32>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut xs: Vec<f32> = Vec::with_capacity(nperturbs);
        let mut ys: Vec<f32> = Vec::with_capacity(nperturbs);
        let scaling = initial_roi.scaling();

        // println!("\ninit: {}", initial_roi);
        for _ in 0..nperturbs {
            let mut roi = initial_roi.clone();
            roi.prepend_scaling_mut(rng.sample(self.distrs.0));
            roi.isometry.translation.vector.x = scaling.mul_add(
                rng.sample(self.distrs.1),
                initial_roi.isometry.translation.vector.x,
            );
            roi.isometry.translation.vector.y = scaling.mul_add(
                rng.sample(self.distrs.1),
                initial_roi.isometry.translation.vector.y,
            );

            // println!("rand: {}", roi);
            let result = self.localize(image, roi);

            xs.push(result.x);
            ys.push(result.y);
        }

        Point2::new(odd_median_mut(&mut xs), odd_median_mut(&mut ys))
    }

    /// Create localizer from a readable source.
    pub fn from_readable(mut readable: impl Read) -> Result<Self, Error> {
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

        let mut stages: Stages = Vec::with_capacity(nstages);

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
            distrs: (Uniform::new(0.925, 0.94), Uniform::new(-0.075, 0.075)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_pupil_localizer_model_parsing() {
        let puploc =
            Localizer::from_readable(include_bytes!("../models/puploc.bin").to_vec().as_slice())
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

        let first_pred_test = Vector2::new(-0.08540829, 0.04436668);
        let last_pred_test = Vector2::new(0.05820565, 0.02249731);
        let first_pred = stages[0][0].1[0];
        let last_pred = stages[stages.len() - 1][trees - 1].1[dsize - 1];
        assert_abs_diff_eq!(first_pred_test, first_pred);
        assert_abs_diff_eq!(last_pred_test, last_pred);
    }
}
