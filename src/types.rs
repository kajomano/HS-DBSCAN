use nalgebra::{Const, Dyn, OMatrix};

pub type FloatType = f64;
pub type IndexType = u16;

pub type MatrixType<const NCOLS: usize> = OMatrix<FloatType, Dyn, Const<NCOLS>>;
