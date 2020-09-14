use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, GrayImage};
use na::{Dynamic, MatrixMN, Point2, Point3, Vector2, U2};

use super::core::ThresholdNode;
use super::geometry::{find_affine, find_similarity};

pub type ShapeMatrix = MatrixMN<f32, U2, Dynamic>;

struct Tree {
    nodes: Vec<ThresholdNode>,
    shifts: Vec<Vec<Vector2<f32>>>,
}

struct Forest {
    trees: Vec<Tree>,
    anchors: Vec<usize>,
    deltas: Vec<Vector2<f32>>,
}

pub struct Shaper {
    initial_shape: Vec<Point2<f32>>,
    forests: Vec<Forest>,
    _dsize: usize,
    _features: usize,
}

impl Shaper {
    pub fn from_readable(mut readable: impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        readable.read_exact(&mut buf[0..1])?;
        let version = buf[0];
        if version != 1 {
            return Err(Error::new(ErrorKind::InvalidData, "wrong version"));
        }

        readable.read_exact(&mut buf)?;
        let nrows = u32::from_be_bytes(buf) as usize;

        readable.read_exact(&mut buf)?;
        let ncols = u32::from_be_bytes(buf) as usize;

        let size = nrows * ncols;

        readable.read_exact(&mut buf)?;
        let nforests = u32::from_be_bytes(buf) as usize;

        readable.read_exact(&mut buf)?;
        let forest_size = u32::from_be_bytes(buf) as usize;

        readable.read_exact(&mut buf)?;
        let tree_depth = u32::from_be_bytes(buf);

        readable.read_exact(&mut buf)?;
        let nfeatures = u32::from_be_bytes(buf) as usize;

        let leafs_count = 2u32.pow(tree_depth) as usize;
        let splits_count = leafs_count - 1;

        // dbg!(nrows, ncols, nforests, forest_size, tree_depth, nfeatures);
        let initial_shape: Vec<Point2<f32>> = shape_from_readable(readable.by_ref(), size)?
            .column_iter()
            .map(|col| Point2::new(col.x, col.y))
            .collect();

        let mut forests: Vec<Forest> = Vec::with_capacity(nforests);
        for _ in 0..nforests {
            let mut trees = Vec::with_capacity(forest_size);
            for _ in 0..forest_size {
                let mut nodes = Vec::with_capacity(splits_count);
                for _ in 0..splits_count {
                    nodes.push(ThresholdNode::from_readable(readable.by_ref())?);
                }

                let mut shifts = Vec::with_capacity(leafs_count);
                for _ in 0..leafs_count {
                    let shift: Vec<Vector2<f32>> = shape_from_readable(readable.by_ref(), size)?
                        .column_iter()
                        .map(|col| Vector2::new(col.x, col.y))
                        .collect();
                    shifts.push(shift);
                }

                trees.push(Tree { nodes, shifts });
            }

            let mut anchors = Vec::with_capacity(nfeatures);
            for _ in 0..nfeatures {
                readable.read_exact(&mut buf)?;
                anchors.push(u32::from_be_bytes(buf) as usize);
            }

            let mut deltas = Vec::with_capacity(nfeatures);
            for _ in 0..nfeatures {
                readable.read_exact(&mut buf)?;
                let x = f32::from_be_bytes(buf);
                readable.read_exact(&mut buf)?;
                let y = f32::from_be_bytes(buf);
                deltas.push(Vector2::new(x, y));
            }

            forests.push(Forest {
                trees,
                anchors,
                deltas,
            });
        }

        Ok(Self {
            initial_shape,
            forests,
            _dsize: splits_count,
            _features: nfeatures,
        })
    }

    pub fn predict(&self, image: &GrayImage, roi: &Point3<f32>) -> Vec<Point2<f32>> {
        let mut shape = self.initial_shape.clone();

        let img_corners = [
            Point2::new(0.0, 0.0),
            Point2::new(image.width() as f32, 0.0),
            Point2::new(image.width() as f32, image.height() as f32),
        ];
        let roi_corners = roi_to_3points(roi);
        let transform_to_image = find_affine(&img_corners, &roi_corners, 0.0001).unwrap();

        let mut features: Vec<u8> = vec![0u8; self._features];

        for forest in self.forests.iter() {
            let transform_to_shape = find_similarity(&self.initial_shape, &shape);

            for ((delta, anchor), feature) in forest
                .deltas
                .iter()
                .zip(forest.anchors.iter())
                .zip(features.iter_mut())
            {
                let mut point = &shape[*anchor] + transform_to_shape.transform_vector(delta);
                point = transform_to_image.transform_point(&point);
                let (x, y) = (point.x as u32, point.y as u32);

                if image.in_bounds(x, y) {
                    *feature = image.get_pixel(x, y).0[0];
                }
            }

            for tree in forest.trees.iter() {
                let mut idx = 0;
                while idx < tree.nodes.len() {
                    idx = 2 * idx + 1 + tree.nodes[idx].bintest(&features) as usize;
                }
                idx = idx.saturating_sub(self._dsize);

                shape.iter_mut().zip(tree.shifts[idx].iter()).for_each(
                    |(shape_point, shift_vector)| {
                        *shape_point += shift_vector;
                    },
                );
            }
        }

        let norm_corners = [
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
        ];

        let transform_to_image = find_affine(&norm_corners, &roi_corners, 0.0001).unwrap();
        shape
            .iter_mut()
            .for_each(|point| *point = transform_to_image.transform_point(point));
        shape
    }
}

fn roi_to_3points(roi: &Point3<f32>) -> [Point2<f32>; 3] {
    let hs = roi.z / 2.0;
    [
        Point2::new(roi.x - hs, roi.y - hs),
        Point2::new(roi.x + hs, roi.y - hs),
        Point2::new(roi.x + hs, roi.y + hs),
    ]
}

fn shape_from_readable(mut readable: impl Read, size: usize) -> Result<ShapeMatrix, Error> {
    let mut arr = Vec::with_capacity(size);
    let mut buf = [0u8; 4];
    for _ in 0..size {
        readable.read_exact(&mut buf)?;
        arr.push(f32::from_be_bytes(buf));
    }
    Ok(ShapeMatrix::from_vec(arr))
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use na::Point3;

    #[test]
    fn check_face_landmarks_model_parsing() {
        let shaper = load_face_landmarks_model();
        assert_eq!(shaper.forests.len(), 15);
        assert_eq!(shaper.forests[0].trees.len(), 500);

        assert_eq!(shaper.forests[0].trees[0].nodes.len(), 15);
        assert_eq!(shaper.forests[0].trees[0].shifts.len(), 16);

        assert_eq!(shaper.forests[0].anchors.len(), 800);
        assert_eq!(shaper.forests[0].deltas.len(), 800);
    }

    #[test]
    fn check_face_landmarks_predict() {
        let shaper = load_face_landmarks_model();
        let (image, (face_roi, _, _)) = load_test_image();
        let face_roi = Point3::new(face_roi.x as f32, face_roi.y as f32, face_roi.z as f32);

        let shape = shaper.predict(&image, &face_roi);
        println!("> predicted shape");

        for (i, point) in shape.iter().enumerate() {
            println!("{}: {}", i, point);
        }
    }
}
