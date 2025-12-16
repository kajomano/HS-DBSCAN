use crate::types::{FloatType, MatrixType};
use nalgebra::{Const, LpNorm, MatrixView, OMatrix, UniformNorm};

type RowVector<'a, const NCOLS: usize> = OMatrix<FloatType, Const<1>, Const<NCOLS>>;
type RowVectorView<'a, const NCOLS: usize> = MatrixView<'a, FloatType, Const<1>, Const<NCOLS>>;

/// Different supported [norms](https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html).
pub enum Norm {
    L1,
    L2,
    L2Squared,
    Linf,
}

impl Norm {
    fn apply<const NCOLS: usize>(&self, a: RowVector<NCOLS>) -> FloatType {
        match self {
            Norm::L1 => a.apply_norm(&LpNorm(1)),
            Norm::L2 => a.norm(),
            Norm::L2Squared => a.norm_squared(),
            Norm::Linf => a.apply_norm(&UniformNorm),
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

    pub fn distance(
        &self,
        anchor: RowVectorView<NCOLS>,
        query: &MatrixType<NCOLS>,
    ) -> Vec<FloatType> {
        query
            .row_iter()
            .map(|query| self.norm.apply(anchor - query))
            .collect()
    }

    pub fn within_proximity(
        &self,
        anchor: RowVectorView<NCOLS>,
        query: &MatrixType<NCOLS>,
    ) -> Vec<bool> {
        query
            .row_iter()
            .map(|query| self.norm.apply(anchor - query) <= self.eps)
            .collect()
    }
}
