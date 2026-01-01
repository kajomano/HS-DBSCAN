use crate::types::{FloatType, MatrixType};
use rand::{Rng, rng};
use rand_distr::{Distribution, Normal};
use std::fmt::{Display, Formatter, Result};

pub trait Generate<const NCOLS: usize>: Display {
    // TODO: docstring
    fn generate(&self, n: usize) -> MatrixType<NCOLS>;
}

pub struct UniformBox<const NCOLS: usize> {
    pub center: [FloatType; NCOLS],
    pub size: FloatType,
}

impl<const NCOLS: usize> Generate<NCOLS> for UniformBox<NCOLS> {
    fn generate(&self, n: usize) -> MatrixType<NCOLS> {
        let mut rng = rng();
        MatrixType::<NCOLS>::from_fn(n, |_, col| {
            rng.random_range(
                (self.center[col] - (self.size / 2.0))..(self.center[col] + (self.size / 2.0)),
            )
        })
    }
}

impl<const NCOLS: usize> Display for UniformBox<NCOLS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "uniformbox")
    }
}

pub struct UniformSphere<const NCOLS: usize> {
    pub center: [FloatType; NCOLS],
    pub radius: FloatType,
}

impl<const NCOLS: usize> Generate<NCOLS> for UniformSphere<NCOLS> {
    fn generate(&self, n: usize) -> MatrixType<NCOLS> {
        let mut rng = rng();
        let normal_dist = Normal::new(0.0, 1.0).unwrap();
        let mut points = MatrixType::<NCOLS>::from_fn(n, |_, _| normal_dist.sample(&mut rng));

        points.row_iter_mut().for_each(|mut row| {
            let u: FloatType = rng.random_range(0.0..1.0);
            let u = u.powf(1.0 / (NCOLS as FloatType)) * self.radius;
            row *= row.norm() * u;
        });

        points
    }
}

impl<const NCOLS: usize> Display for UniformSphere<NCOLS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "uniformsphere")
    }
}
