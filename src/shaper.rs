use std::io::{Error, ErrorKind, Read};

use image::{GenericImageView, Luma};
use imageproc::rect::Rect;
use nalgebra::allocator::Allocator;
use nalgebra::{
    Affine2, DefaultAllocator, Dim, DimName, Dyn, Matrix3, OMatrix, Point2, SimilarityMatrix2,
    UninitMatrix, Vector2, U2,
};

use crate::nodes::ThresholdNode;
use crate::utils::img::get_luma_by_point_f32;

struct Tree {
    nodes: Vec<ThresholdNode>,
    shifts: Vec<Vec<Vector2<f32>>>,
}

struct Delta {
    anchor: usize,
    value: Vector2<f32>,
}

struct Forest {
    trees: Vec<Tree>,
    deltas: Vec<Delta>,
}

#[inline]
fn extract_features<I>(
    deltas: &[Delta],
    image: &I,
    transform_to_shape: &SimilarityMatrix2<f32>,
    transform_to_image: &Affine2<f32>,
    shape: &[Point2<f32>],
) -> Vec<u8>
where
    I: GenericImageView<Pixel = Luma<u8>>,
{
    deltas
        .iter()
        .map(|delta| {
            let point = unsafe { shape.get_unchecked(delta.anchor) };
            let point = point + transform_to_shape.transform_vector(&delta.value);
            let point = transform_to_image * point;

            get_luma_by_point_f32(image, point).unwrap_or(0u8)
        })
        .collect()
}

/// Implements object alignment using an ensemble of regression trees.
pub struct Shaper {
    depth: usize,
    dsize: usize,
    shape: Vec<Point2<f32>>,
    forests: Vec<Forest>,
}

impl Shaper {
    #[inline]
    pub fn size(&self) -> usize {
        self.shape.len()
    }

    #[inline]
    pub fn init_points(&self) -> &[Point2<f32>] {
        self.shape.as_ref()
    }

    /// Create a shaper object from a readable source.
    #[inline]
    pub fn load(mut readable: impl Read) -> Result<Self, Error> {
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

        let size = nrows * ncols / U2::USIZE;

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
        let shape: Vec<Point2<f32>> = read_shape(readable.by_ref(), size)?
            .column_iter()
            .map(|col| Point2::new(col.x, col.y))
            .collect();

        let mut forests: Vec<Forest> = Vec::with_capacity(nforests);
        for _ in 0..nforests {
            let mut trees = Vec::with_capacity(forest_size);
            for _ in 0..forest_size {
                let mut nodes = Vec::with_capacity(splits_count);
                let mut buf10 = [0u8; 10];
                for _ in 0..splits_count {
                    readable.read_exact(&mut buf10)?;
                    nodes.push(ThresholdNode::from(buf10));
                }

                let mut shifts = Vec::with_capacity(leafs_count);
                for _ in 0..leafs_count {
                    let shift: Vec<Vector2<f32>> = read_shape(readable.by_ref(), size)?
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
            for anchor in anchors.into_iter() {
                readable.read_exact(&mut buf)?;
                let x = f32::from_be_bytes(buf);

                readable.read_exact(&mut buf)?;
                let y = f32::from_be_bytes(buf);

                deltas.push(Delta {
                    anchor,
                    value: Vector2::new(x, y),
                });
            }

            forests.push(Forest { trees, deltas });
        }

        Ok(Self {
            depth: tree_depth as usize,
            dsize: splits_count,
            shape,
            forests,
        })
    }

    /// Estimate object shape on the image
    ///
    /// ### Arguments
    ///
    /// * `image` - Target image.
    /// TODO:
    ///
    /// ### Returns
    ///
    /// A collection of points each one corresponds to landmark location.
    /// Points count is defined by a loaded shaper model.
    #[inline]
    pub fn shape<I>(&self, image: &I, rect: Rect) -> Vec<Point2<f32>>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        let mut shape = self.shape.clone();

        let transform_to_image = find_transform_to_image(rect);

        for forest in self.forests.iter() {
            let transform_to_shape = similarity_least_squares::from_point_slices(
                self.shape.as_slice(),
                shape.as_slice(),
                f32::EPSILON,
                0,
            )
            .expect("Similarity least squares failed");

            let features = extract_features(
                &forest.deltas,
                image,
                &transform_to_shape,
                &transform_to_image,
                &shape,
            );

            for tree in forest.trees.iter() {
                let idx = (0..self.depth).fold(0, |idx, _| {
                    2 * idx + 1 + tree.nodes[idx].bintest(features.as_slice()) as usize
                }) - self.dsize;

                shape.iter_mut().zip(tree.shifts[idx].iter()).for_each(
                    |(shape_point, shift_vector)| {
                        *shape_point += shift_vector;
                    },
                );
            }
        }

        shape
            .iter_mut()
            .for_each(|point| *point = transform_to_image * *point);
        shape
    }
}

#[inline]
fn find_transform_to_image(rect: Rect) -> Affine2<f32> {
    Affine2::from_matrix_unchecked(Matrix3::new(
        rect.width() as f32,
        0.0,
        rect.left() as f32,
        0.0,
        rect.height() as f32,
        rect.top() as f32,
        0.0,
        0.0,
        1.0,
    ))
}

#[inline]
fn read_shape(mut readable: impl Read, size: usize) -> Result<OMatrix<f32, U2, Dyn>, Error>
where
    DefaultAllocator: Allocator<f32, U2, Dyn>,
{
    let mut m = UninitMatrix::<f32, U2, Dyn>::uninit(U2, Dyn::from_usize(size));

    let mut buf = [0u8; 4];
    for value in m.iter_mut() {
        readable.read_exact(&mut buf)?;
        value.write(f32::from_be_bytes(buf));
    }
    Ok(unsafe { m.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_landmarks_model_loading() {
        let shaper = Shaper::load(
            include_bytes!("../models/face-5.shaper.bin")
                .to_vec()
                .as_slice(),
        )
        .expect("parsing failed");

        assert_eq!(shaper.forests.len(), 15);
        assert_eq!(shaper.forests[0].trees.len(), 500);

        assert_eq!(shaper.forests[0].trees[0].nodes.len(), 15);
        assert_eq!(shaper.forests[0].trees[0].shifts.len(), 16);

        assert_eq!(shaper.forests[0].deltas.len(), 800);
    }
}
