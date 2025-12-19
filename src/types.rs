use nalgebra::{Const, Dyn, Matrix, OMatrix, Storage};

pub type FloatType = f64;
pub type IndexType = u16;

pub type MatrixType<const NCOLS: usize> = OMatrix<FloatType, Dyn, Const<NCOLS>>;
// NOTE: the bound on S is not enforced through type aliases, so add it as a bound on the function too!
#[allow(type_alias_bounds)]
pub type MatrixViewType<const NCOLS: usize, S: Storage<FloatType, Dyn, Const<NCOLS>>> =
    Matrix<FloatType, Dyn, Const<NCOLS>, S>;
