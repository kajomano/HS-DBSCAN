use nalgebra::{Const, Dyn, OMatrix, OVector};

pub type FloatType = f32;
pub type IndexType = u16;
pub type RasterType = i16;

pub type FloatMatrixType<const NDIMS: usize> = OMatrix<FloatType, Dyn, Const<NDIMS>>;
pub type IndexVectorType = OVector<IndexType, Dyn>;
