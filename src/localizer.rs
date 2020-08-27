use std::io::{Error, ErrorKind, Read};

use super::core::{create_leaf_transform, Bintest, ComparisonNode};
use image::GrayImage;
use na::{Point2, Point3, Vector2};

use rand::distributions::Uniform;
use rand::{Rng, RngCore};

type Tree = Vec<ComparisonNode>;
type Predictions = Vec<Point2<f32>>;
type Stage = Vec<(Tree, Predictions)>;
type Stages = Vec<Stage>;

pub struct Localizer {
    depth: usize,
    dsize: usize,
    scale: f32,
    stages: Stages,
    distrs: (Uniform<f32>, Uniform<f32>),
}

impl Localizer {
    pub fn localize(&self, image: &GrayImage, roi: &Point3<f32>) -> Point2<f32> {
        let mut roi = roi.clone();
        self.localize_mut(image, &mut roi);
        roi.xy()
    }

    pub fn localize_mut(&self, image: &GrayImage, point: &mut Point3<f32>) {
        for stage in self.stages.iter() {
            let mut dvec = Vector2::new(0.0, 0.0);

            for (codes, preds) in stage.iter() {
                let transform = create_leaf_transform(&point);

                let idx = (0..self.depth).fold(0, |idx, _| {
                    let bintest = codes[idx].bintest(image, &transform);
                    2 * idx + 1 + bintest as usize
                });
                let lutidx = idx.saturating_sub(self.dsize) + 1;

                let pred = &preds[lutidx];

                dvec.x += pred.x;
                dvec.y += pred.y;
            }

            point.x += dvec.x * point.z;
            point.y += dvec.y * point.z;

            point.z *= self.scale;
        }
    }

    pub fn perturb_localize(
        &self,
        image: &GrayImage,
        roi: &Point3<f32>,
        mut rng: impl RngCore,
        nperturbs: usize,
    ) -> Point2<f32> {
        let mut xs: Vec<f32> = Vec::with_capacity(nperturbs);
        let mut ys: Vec<f32> = Vec::with_capacity(nperturbs);

        // println!("\ninit: {}", roi);
        for _ in 0..nperturbs {
            let z = rng.sample(self.distrs.0) * roi.z;
            let x = roi.z.mul_add(rng.sample(self.distrs.1), roi.x);
            let y = roi.z.mul_add(rng.sample(self.distrs.1), roi.y);
            let mut _roi = Point3::new(x, y, z);

            // println!("rand: {}", _roi);
            self.localize_mut(image, &mut _roi);

            xs.push(_roi.x);
            ys.push(_roi.y);
        }

        Point2::new(odd_median_mut(&mut xs), odd_median_mut(&mut ys))
    }

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

                    predictions.push(Point2::new(x, y));
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

pub fn odd_median_mut(numbers: &mut Vec<f32>) -> f32 {
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    numbers[numbers.len() >> 1]
}

#[cfg(test)]
mod tests {
    use image::DynamicImage;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    use super::*;

    fn load_model() -> Localizer {
        let path = Path::new("./models/puploc.bin");
        let fp = File::open(path).unwrap();
        Localizer::from_readable(fp).unwrap()
    }

    fn load_test_image(path: &Path) -> GrayImage {
        match image::open(path).unwrap() {
            DynamicImage::ImageLuma8(image) => image,
            _ => panic!("invalid test image"),
        }
    }

    fn create_init_point(point: &Point2<f32>) -> Point3<f32> {
        let mut init_point = point.xyx();
        init_point.x += 5.0;
        init_point.y += 5.0;
        init_point.z = 40.0;
        init_point
    }

    fn load_test_data(path: &Path) -> (Point2<f32>, Point2<f32>) {
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);

        let mut buf = String::new();
        reader.read_line(&mut buf).expect("no first line");
        buf.clear();

        reader.read_line(&mut buf).expect("no data");
        let data = buf
            .trim()
            .split("\t")
            .filter_map(|s| s.parse::<f32>().ok())
            .collect::<Vec<_>>();

        (Point2::new(data[0], data[1]), Point2::new(data[2], data[3]))
    }

    #[test]
    fn check_pupil_localizer_model_parsing() {
        let puploc = load_model();
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

        let first_pred_test = Point2::new(-0.08540829, 0.04436668);
        let last_pred_test = Point2::new(0.05820565, 0.02249731);
        let first_pred = stages[0][0].1[0];
        let last_pred = stages[stages.len() - 1][trees - 1].1[dsize - 1];
        assert_abs_diff_eq!(first_pred_test, first_pred);
        assert_abs_diff_eq!(last_pred_test, last_pred);
    }

    #[test]
    fn check_pupil_localization() {
        let assets_dir = Path::new("./assets/");

        let image_path = assets_dir.join("Lenna_grayscale_test.jpg");
        let puploc = load_model();
        let image = load_test_image(&image_path);
        let (left_pupil, right_pupil) = load_test_data(&image_path.with_extension("txt"));

        let epsilon = 3f32;
        let pred = puploc.localize(&image, &create_init_point(&left_pupil));
        assert_abs_diff_eq!(left_pupil, pred, epsilon = epsilon);

        let pred = puploc.localize(&image, &create_init_point(&right_pupil));
        assert_abs_diff_eq!(right_pupil, pred, epsilon = epsilon);
    }

    #[test]
    fn check_perturbated_pupil_localization() {
        let assets_dir = Path::new("./assets/");

        let image_path = assets_dir.join("Lenna_grayscale_test.jpg");
        let puploc = load_model();
        let image = load_test_image(&image_path);
        let (left_pupil, right_pupil) = load_test_data(&image_path.with_extension("txt"));

        let mut rng = XorShiftRng::seed_from_u64(42u64);

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
