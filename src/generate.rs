use crate::types::{FloatType, MatrixType};
use rand::{Rng, rng};
use std::fmt::{Display, Formatter, Result};

pub trait Generate<const NCOLS: usize>: Display {
    fn generate(&self, n_points: usize) -> MatrixType<NCOLS>;
}

pub struct Uniform {
    pub extent: FloatType,
}

impl<const NCOLS: usize> Generate<NCOLS> for Uniform {
    fn generate(&self, n_points: usize) -> MatrixType<NCOLS> {
        let mut rng = rng();
        MatrixType::<NCOLS>::from_fn(n_points as usize, |_, _| {
            rng.random_range(-self.extent..self.extent)
        })
    }
}

impl Display for Uniform {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "uniform")
    }
}
