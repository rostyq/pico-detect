use std::fmt::Debug;
use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};
use nalgebra::{Point2, Translation2, Vector2};

use crate::geometry::Target;
use crate::nodes::ComparisonNode;

type Tree = Vec<ComparisonNode>;
type Predictions = Vec<Vector2<f32>>;
type Stage = Vec<(Tree, Predictions)>;

/// Implements object localization using decision trees.
///
/// Details available [here](https://tehnokv.com/posts/puploc-with-trees/).
#[derive(Clone)]
pub struct Localizer {
    depth: usize,
    dsize: usize,
    scale: f32,
    stages: Vec<Stage>,
}

impl Debug for Localizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(Localizer))
            .field("depth", &self.depth)
            .field("dsize", &self.dsize)
            .field("scale", &self.scale)
            .field("stages", &self.stages.len())
            .finish()
    }
}

impl Localizer {
    /// Estimate object location on the image
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// TODO
    #[inline]
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
            let p = unsafe { point.coords.try_cast::<i32>().unwrap_unchecked() }.into();
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
    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pupil_localizer_model_loading() {
        let puploc = dbg!(Localizer::load(
            include_bytes!("../../../models/pupil.localizer.bin")
                .to_vec()
                .as_slice(),
        )
        .expect("parsing failed"));

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
