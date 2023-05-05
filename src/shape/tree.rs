use std::io::{Error, Read};

use nalgebra::Vector2;

use crate::nodes::ThresholdNode;

use super::utils::read_shape;

#[derive(Debug, Clone)]
pub struct ShaperTree {
    nodes: Vec<ThresholdNode>,
    shifts: Vec<Vec<Vector2<f32>>>,
}

impl ShaperTree {
    #[cfg(test)]
    pub fn nodes(&self) -> usize {
        self.nodes.len()
    }

    #[cfg(test)]
    pub fn shifts(&self) -> usize {
        self.shifts.len()
    }

    #[inline]
    pub fn node(&self, index: usize) -> &ThresholdNode {
        unsafe { self.nodes.get_unchecked(index) }
    }

    #[inline]
    pub fn shift(&self, index: usize) -> &Vec<Vector2<f32>> {
        unsafe { self.shifts.get_unchecked(index) }
    }

    #[inline(always)]
    fn load_nodes<R: Read>(mut reader: R, count: usize) -> Result<Vec<ThresholdNode>, Error> {
        let mut nodes = Vec::with_capacity(count);
        let mut buf10 = [0u8; 10];

        for _ in 0..count {
            reader.read_exact(&mut buf10)?;
            nodes.push(ThresholdNode::from(buf10));
        }

        Ok(nodes)
    }

    #[inline(always)]
    fn load_shifts<R: Read>(mut reader: R, count: usize, size: usize) -> Result<Vec<Vec<Vector2<f32>>>, Error> {
        let mut shifts = Vec::with_capacity(count);

        for _ in 0..count {
            let shift: Vec<Vector2<f32>> = read_shape(reader.by_ref(), size)?
                .column_iter()
                .map(|col| Vector2::new(col.x, col.y))
                .collect();
            shifts.push(shift);
        }

        Ok(shifts)
    }

    #[inline]
    pub fn load<R: Read>(mut reader: R, nodes: usize, shifts: usize, shape: usize) -> Result<Self, Error> {
        Ok(Self {
            nodes: Self::load_nodes(reader.by_ref(), nodes)?,
            shifts: Self::load_shifts(reader.by_ref(), shifts, shape)?,
        })
    }
}
