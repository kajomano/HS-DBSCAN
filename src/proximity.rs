use crate::types::FloatType;
use nalgebra::{Const, LpNorm, OVector, UniformNorm, VectorView};

type VectorRef<'a, const NCOLS: usize> = VectorView<'a, FloatType, Const<NCOLS>>;

/// Different supported [norms](https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html]).
pub enum Norm {
    L1,
    L2,
    L2Squared,
    Linf,
}

impl Norm {
    fn apply<const NCOLS: usize>(&self, vec: OVector<FloatType, Const<NCOLS>>) -> FloatType {
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

    pub fn distance(&self, a: VectorRef<NCOLS>, b: VectorRef<NCOLS>) -> FloatType {
        self.norm.apply(a - b)
    }

    pub fn within_proximity(&self, a: VectorRef<NCOLS>, b: VectorRef<NCOLS>) -> bool {
        self.distance(a, b) <= self.eps
    }
}
