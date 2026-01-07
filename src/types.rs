use nalgebra::{Const, Dyn, OMatrix, OVector};

pub type FloatType = f32;
pub type IndexType = u16;
pub type RasterType = i16;

pub type MatrixType<const NDIMS: usize, T> = OMatrix<T, Const<NDIMS>, Dyn>;
pub type FloatMatrixType<const NDIMS: usize> = MatrixType<NDIMS, FloatType>;
pub type IndexVectorType = OVector<IndexType, Dyn>;
