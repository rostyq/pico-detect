use std::io::{Error, ErrorKind, Read};

use image::GrayImage;
use na::geometry::{Similarity2, Translation2, UnitComplex};
use na::{Point2, Point3, Vector2, Vector3};

use rand::distributions::Uniform;
use rand::{Rng, RngCore};

use super::core::{Bintest, ComparisonNode, SaturatedGet};
use super::geometry::scale_and_translate_fast;

type Tree = Vec<ComparisonNode>;
type Predictions = Vec<Vector2<f32>>;
type Stage = Vec<(Tree, Predictions)>;
type Stages = Vec<Stage>;

impl Bintest<Similarity2<f32>> for ComparisonNode {
    #[inline]
    fn find_point(transform: &Similarity2<f32>, point: &Point2<i8>) -> Point2<u32> {
        scale_and_translate_fast(
            point,
            &Vector3::new(
                transform.isometry.translation.x.round() as i32,
                transform.isometry.translation.y.round() as i32,
                transform.scaling() as i32,
            ),
        )
    }

    #[inline]
    fn find_lum(image: &GrayImage, transform: &Similarity2<f32>, point: &Point2<i8>) -> u8 {
        let point = Self::find_point(transform, point);
        image.saturated_get_lum(point.x, point.y)
    }

    #[inline]
    fn bintest(&self, image: &GrayImage, transform: &Similarity2<f32>) -> bool {
        let lum0 = Self::find_lum(image, transform, &self.left);
        let lum1 = Self::find_lum(image, transform, &self.right);
        lum0 > lum1
    }
}

#[inline]
fn create_leaf_transform(point: &Point3<f32>) -> Similarity2<f32> {
    Similarity2::from_parts(
        Translation2::new(point.x, point.y),
        UnitComplex::identity(),
        point.z,
    )
}

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
    /// * `roi` - Initial location to start:
    ///   - `roi.x` position on image x-axis,
    ///   - `roi.y` position on image y-axis,
    ///   - `roi.z` initial window size to search.
    pub fn localize(&self, image: &GrayImage, roi: &Point3<f32>) -> Point2<f32> {
        let mut transform = create_leaf_transform(&roi);

        for stage in self.stages.iter() {
            let mut translation = Translation2::identity();

            for (codes, preds) in stage.iter() {
                let idx = (0..self.depth).fold(0, |idx, _| {
                    2 * idx
                        + 1
                        + unsafe { codes.get_unchecked(idx) }.bintest(image, &transform) as usize
                });
                let lutidx = idx.saturating_sub(self.dsize) + 1;

                translation.vector += preds[lutidx];
            }

            translation.vector.scale_mut(roi.z);
            transform.append_translation_mut(&translation);

            transform.prepend_scaling_mut(self.scale);
        }

        Point2::from(transform.isometry.translation.vector)
    }

    /// Estimate object location on the image with perturbation to increase accuracy.
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// * `roi` - Initial location to start:
    ///   - `roi.x` initial position on image x-axis,
    ///   - `roi.y` initial position on image y-axis,
    ///   - `roi.z` initial window size (in pixels) to search.
    /// * `rng` - Source for randomness.
    /// * `nperturbs` - How many perturbations to make.
    pub fn perturb_localize(
        &self,
        image: &GrayImage,
        roi: &Point3<f32>,
        mut rng: impl RngCore,
        nperturbs: usize,
    ) -> Point2<f32> {
        let mut xs: Vec<f32> = Vec::with_capacity(nperturbs);
        let mut ys: Vec<f32> = Vec::with_capacity(nperturbs);
        let mut point = *roi;

        // println!("\ninit: {}", roi);
        for _ in 0..nperturbs {
            point.z = rng.sample(self.distrs.0) * roi.z;
            point.x = roi.z.mul_add(rng.sample(self.distrs.1), roi.x);
            point.y = roi.z.mul_add(rng.sample(self.distrs.1), roi.y);

            // println!("rand: {}", _roi);
            let result = self.localize(image, &point);

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
                    let node = ComparisonNode::from_buffer(&buffer);
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

fn odd_median_mut(numbers: &mut Vec<f32>) -> f32 {
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    numbers[numbers.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

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

        let first_node = ComparisonNode::new([30, -16, 125, 14]);
        let last_node = ComparisonNode::new([-125, 26, 15, 98]);
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

    #[test]
    fn bintest_image_edges() {
        let (width, height) = (255, 255);
        let mut image = GrayImage::new(width, height);
        image.put_pixel(0, 0, Luma::from([42u8]));
        image.put_pixel(width - 1, height - 1, Luma::from([255u8]));
        let node = ComparisonNode::new([i8::MAX, i8::MAX, i8::MIN, i8::MIN]);

        let point = Point3::new((width as f32) / 2.0, (height as f32) / 2.0, width as f32);
        let transform = create_leaf_transform(&point);
        let result = node.bintest(&image, &transform);
        assert!(result);
    }
}
