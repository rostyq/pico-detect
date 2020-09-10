use std::io::{Error, ErrorKind, Read};

use image::{GrayImage, GenericImageView};
use na::{Dynamic, MatrixMN, Vector2, Point3, U2};

use super::core::ThresholdNode;

pub type Shape = MatrixMN<f32, U2, Dynamic>;

struct Tree {
    nodes: Vec<ThresholdNode>,
    shapes: Vec<Shape>,
}

struct Forest {
    trees: Vec<Tree>,
    anchors: Vec<u32>,
    deltas: Vec<Vector2<f32>>,
}

pub struct Shaper {
    initial_shape: Shape,
    forests: Vec<Forest>,
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
        let initial_shape = shape_from_readable(readable.by_ref(), size)?;
        // println!("> initial shape\n{}", initial_shape);

        let mut forests: Vec<Forest> = Vec::with_capacity(nforests);
        for _ in 0..nforests {

            let mut trees = Vec::with_capacity(forest_size);
            for _ in 0..forest_size {

                let mut nodes = Vec::with_capacity(splits_count);
                for _ in 0..splits_count {
                    nodes.push(ThresholdNode::from_readable(readable.by_ref())?);
                }

                let mut shapes = Vec::with_capacity(leafs_count);
                for _ in 0..leafs_count {
                    shapes.push(shape_from_readable(readable.by_ref(), size)?);
                }

                trees.push( Tree { nodes, shapes } );
            }

            let mut anchors = Vec::with_capacity(nfeatures);
            for _ in 0..nfeatures {
                readable.read_exact(&mut buf)?;
                anchors.push(u32::from_be_bytes(buf));
            }

            let mut deltas = Vec::with_capacity(nfeatures);
            for _ in 0..nfeatures {
                readable.read_exact(&mut buf)?;
                let x = f32::from_be_bytes(buf);
                readable.read_exact(&mut buf)?;
                let y = f32::from_be_bytes(buf);
                deltas.push(Vector2::new(x, y));
            }

            forests.push(Forest { trees, anchors, deltas });
        }

        Ok(Self { initial_shape, forests, })
    }

    pub fn predict(&self, image: &GrayImage, roi: &Point3<f32>) -> Shape {
        let mut shape = self.initial_shape.clone();
        let mut features: Vec<f32> = Vec::new();

        for forest in self.forests.iter() {

        }

        shape
    }
}

fn shape_from_readable(mut readable: impl Read, size: usize) -> Result<Shape, Error> {
    let mut arr = Vec::with_capacity(size);
    let mut buf = [0u8; 4];
    for _ in 0..size {
        readable.read_exact(&mut buf)?;
        arr.push(f32::from_be_bytes(buf));
    }
    Ok(Shape::from_vec(arr))
}

#[cfg(test)]
mod tests {
    use na::Point3;
    use crate::test_utils::*;

    #[test]
    fn check_face_landmarks_model_parsing() {
        let shaper = load_face_landmarks_model();
        assert_eq!(shaper.forests.len(), 15);
        assert_eq!(shaper.forests[0].trees.len(), 500);

        assert_eq!(shaper.forests[0].trees[0].nodes.len(), 15);
        assert_eq!(shaper.forests[0].trees[0].shapes.len(), 16);

        assert_eq!(shaper.forests[0].anchors.len(), 800);
        assert_eq!(shaper.forests[0].deltas.len(), 800);
    }

    #[test]
    fn check_face_landmarks_predict() {
        let shaper = load_face_landmarks_model();
        let (image, (face_roi, _, _)) = load_test_image();
        let face_roi = Point3::new(
            face_roi.x as f32,
            face_roi.y as f32,
            face_roi.z as f32
        );

        let shape = shaper.predict(&image, &face_roi);
        println!("> predicted shape\n{}", shape);
    }
}
