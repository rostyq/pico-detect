use std::io::{Error, ErrorKind, Read};

use super::core::{create_leaf_transform, ComparisonNode, Bintest};
use image::GrayImage;
use na::{Point2, Point3, Translation2, Vector2};

use rand::distributions::Uniform;
use rand::{Rng, RngCore};

type Tree = Vec<ComparisonNode>;
type Predictions = Vec<Vector2<f32>>;
type Stage = Vec<(Tree, Predictions)>;
type Stages = Vec<Stage>;

/// Implements object localization using decision trees.
///
/// Details available [here](https://tehnokv.com/posts/puploc-with-trees/).
///
/// ### Example
/// ```rust
/// use std::fs::File;
/// use image::{DynamicImage, Rgba};
/// use nalgebra::Point3;
/// use pico_detect::{Localizer, create_xorshift_rng};
///
/// // load pupil localizer
/// let fp = File::open("./models/puploc.bin").unwrap();
/// let puploc = Localizer::from_readable(fp).unwrap();
///
/// // load image
/// let gray = match image::open("./assets/Lenna_grayscale_test.jpg").unwrap() {
///      DynamicImage::ImageLuma8(image) => image,
///      _ => panic!("image loading failed"),
/// };
///
/// // initial parameters
/// let mut rng = create_xorshift_rng(42u64);
/// let init_point = Point3::new(270f32, 270f32, 40f32);
/// let nperturbs = 31usize;
///
/// // find pupil center
/// let pupil_point = puploc.perturb_localize(
///   &gray,
///   &init_point,
///   &mut rng,
///   nperturbs
/// );
///
/// // draw red cross on the image
/// let mut palette = Vec::with_capacity(255);
/// for i in 0..255 {
///     palette.push((i, i, i))
/// }
/// let mut image = gray.expand_palette(&palette, None);
/// let x = pupil_point.x.round() as i32;
/// let y = pupil_point.y.round() as i32;
/// for i in -1..2 {
///     for j in -1..2 {
///         if i == 0 || j == 0 {
///             image.put_pixel((x + i) as u32, (y + j) as u32, Rgba([255, 0, 0, 0]));
///         }
///     }
/// }
/// image.save("./result.jpg");
/// ```
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
                    2 * idx + 1 + unsafe { codes.get_unchecked(idx) }.bintest(image, &transform) as usize
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
        let mut point = roi.clone();

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
    use crate::create_xorshift_rng;
    use crate::test_utils::*;

    #[test]
    fn check_pupil_localizer_model_parsing() {
        let puploc = load_puploc_model();
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
    fn check_pupil_localization() {
        let puploc = load_puploc_model();
        let (image, (left_pupil, right_pupil)) = load_test_image();

        let epsilon = 3f32;
        let pred = puploc.localize(&image, &create_init_point(&left_pupil));
        assert_abs_diff_eq!(left_pupil, pred, epsilon = epsilon);

        let pred = puploc.localize(&image, &create_init_point(&right_pupil));
        assert_abs_diff_eq!(right_pupil, pred, epsilon = epsilon);
    }

    #[test]
    fn check_perturbated_pupil_localization() {
        let puploc = load_puploc_model();
        let (image, (left_pupil, right_pupil)) = load_test_image();

        let mut rng = create_xorshift_rng(42u64);

        let epsilon = 1.5f32;
        let nperturbs = 31usize;
        let pred =
            puploc.perturb_localize(&image, &create_init_point(&left_pupil), &mut rng, nperturbs);
        assert_abs_diff_eq!(left_pupil, pred, epsilon = epsilon);

        let pred = puploc.perturb_localize(
            &image,
            &create_init_point(&right_pupil),
            &mut rng,
            nperturbs,
        );
        assert_abs_diff_eq!(right_pupil, pred, epsilon = epsilon);
    }
}
