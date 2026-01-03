use nalgebra::{Const, Dyn, OMatrix, OVector};

pub type FloatType = f64;
pub type IndexType = u16;
pub type RasterType = i16;

pub type FloatMatrixType<const NCOLS: usize> = OMatrix<FloatType, Dyn, Const<NCOLS>>;
pub type IndexVectorType = OVector<IndexType, Dyn>;
