use std::io::{Error, Read};

use nalgebra::{allocator::Allocator, DefaultAllocator, Dim, Dyn, OMatrix, UninitMatrix, U2};

#[inline]
pub fn read_shape<R: Read>(mut reader: R, size: usize) -> Result<OMatrix<f32, U2, Dyn>, Error>
where
    DefaultAllocator: Allocator<f32, U2, Dyn>,
{
    let mut m = UninitMatrix::<f32, U2, Dyn>::uninit(U2, Dyn::from_usize(size));

    let mut buf = [0u8; 4];

    for value in m.iter_mut() {
        reader.read_exact(&mut buf)?;
        value.write(f32::from_be_bytes(buf));
    }

    Ok(unsafe { m.assume_init() })
}
