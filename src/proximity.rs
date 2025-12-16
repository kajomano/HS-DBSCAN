use crate::types::{FloatType, MatrixType};
use nalgebra::{Const, LpNorm, Matrix, OMatrix, Storage, UniformNorm};

type RowVectorType<const NCOLS: usize> = OMatrix<FloatType, Const<1>, Const<NCOLS>>;
type RowVectorViewType<const NCOLS: usize, S> = Matrix<FloatType, Const<1>, Const<NCOLS>, S>;

/// Different supported [norms](https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html).
pub enum Norm {
    L1,
    L2,
    L2Squared,
    Linf,
}

impl Norm {
    fn apply<const NCOLS: usize>(&self, vec: RowVectorType<NCOLS>) -> FloatType {
        match self {
            Norm::L1 => vec.apply_norm(&LpNorm(1)),
            Norm::L2 => vec.norm(),
            Norm::L2Squared => vec.norm_squared(),
            Norm::Linf => vec.apply_norm(&UniformNorm),
        }
    }
}

pub struct Proximity<const NCOLS: usize> {
    eps: FloatType,
    norm: Norm,
}

impl<const NCOLS: usize> Proximity<NCOLS> {
    pub fn new(eps: FloatType, norm: Norm) -> Self {
        Self {
            eps: match norm {
                Norm::L2Squared => eps * eps,
                _ => eps,
            },
            norm,
        }
    }

    pub fn distance<S>(
        &self,
        anchor: &RowVectorViewType<NCOLS, S>,
        query: &MatrixType<NCOLS>,
    ) -> Vec<FloatType>
    where
        S: Storage<FloatType, Const<1>, Const<NCOLS>>,
    {
        query
            .row_iter()
            .map(|query| self.norm.apply(anchor - query))
            .collect()
    }

    pub fn within_proximity<S>(
        &self,
        anchor: &RowVectorViewType<NCOLS, S>,
        query: &MatrixType<NCOLS>,
    ) -> Vec<bool>
    where
        S: Storage<FloatType, Const<1>, Const<NCOLS>>,
    {
        query
            .row_iter()
            .map(|query| self.norm.apply(anchor - query) <= self.eps)
            .collect()
    }
}
