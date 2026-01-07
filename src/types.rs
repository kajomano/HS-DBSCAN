use nalgebra::{Const, Dyn, OMatrix, OVector};

pub type FloatType = f32;
pub type IndexType = u16;
pub type RasterType = i16;

pub type FloatMatrixType<const NDIMS: usize> = OMatrix<FloatType, Const<NDIMS>, Dyn>;
pub type IndexVectorType = OVector<IndexType, Dyn>;
