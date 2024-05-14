use core::mem::size_of;
use std::io::{Error, Read};

use image::{GenericImageView, Luma};
use nalgebra::{Affine2, Point2, SimilarityMatrix2};
use pixelutil_image::get_pixel;

use super::delta::ShaperDelta;
use super::tree::ShaperTree;

#[derive(Debug, Clone)]
pub struct ShaperForest {
    deltas: Vec<ShaperDelta>,
    trees: Vec<ShaperTree>,
}

impl ShaperForest {
    #[cfg(test)]
    pub fn trees(&self) -> usize {
        self.trees.len()
    }

    #[cfg(test)]
    pub fn deltas(&self) -> usize {
        self.deltas.len()
    }

    #[cfg(test)]
    pub fn tree(&self, index: usize) -> &ShaperTree {
        self.trees.get(index).unwrap()
    }

    #[inline]
    pub fn trees_slice(&self) -> &[ShaperTree] {
        &self.trees
    }

    #[inline]
    pub(super) fn extract_features<I>(
        &self,
        image: &I,
        transform_to_shape: &SimilarityMatrix2<f32>,
        transform_to_image: &Affine2<f32>,
        shape: &[Point2<f32>],
    ) -> Vec<u8>
    where
        I: GenericImageView<Pixel = Luma<u8>>,
    {
        self.deltas
            .iter()
            .map(|delta| {
                let point = unsafe { shape.get_unchecked(delta.anchor()) };
                let point = point + transform_to_shape.transform_vector(delta.value());
                let point = transform_to_image * point;
                let point = unsafe { point.coords.try_cast::<i32>().unwrap_unchecked() };

                get_pixel(image, point.x, point.y).map(|p| p.0[0]).unwrap_or(0u8)
            })
            .collect()
    }

    #[inline]
    fn load_trees<R: Read>(
        mut reader: R,
        count: usize,
        nodes: usize,
        shifts: usize,
        shape: usize,
    ) -> Result<Vec<ShaperTree>, Error> {
        let mut trees = Vec::with_capacity(count);

        for _ in 0..count {
            trees.push(ShaperTree::load(reader.by_ref(), nodes, shifts, shape)?);
        }

        Ok(trees)
    }

    #[inline]
    fn load_anchors<R: Read>(mut reader: R, count: usize) -> Result<Vec<usize>, Error> {
        let mut buf = [0u8; size_of::<u32>()];

        let mut anchors = Vec::with_capacity(count);

        for _ in 0..count {
            reader.read_exact(&mut buf)?;
            anchors.push(u32::from_be_bytes(buf) as usize);
        }

        Ok(anchors)
    }

    #[inline]
    pub(super) fn load<R: Read>(
        mut reader: R,
        size: usize,
        nodes: usize,
        shifts: usize,
        shape: usize,
        features: usize,
    ) -> Result<Self, Error> {
        let trees = Self::load_trees(reader.by_ref(), size, nodes, shifts, shape)?;

        let mut buf = [0u8; size_of::<f32>()];
        let mut deltas = Vec::with_capacity(features);

        for anchor in Self::load_anchors(reader.by_ref(), features)?.into_iter() {
            reader.read_exact(&mut buf)?;
            let x = f32::from_be_bytes(buf);

            reader.read_exact(&mut buf)?;
            let y = f32::from_be_bytes(buf);

            deltas.push(ShaperDelta::new(anchor, x, y));
        }

        Ok(Self { trees, deltas })
    }
}
