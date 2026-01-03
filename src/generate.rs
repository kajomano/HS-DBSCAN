use crate::types::{FloatMatrixType, FloatType};
use rand::{Rng, rng};
use rand_distr::{Distribution, Normal};
use std::fmt::{Display, Formatter, Result};

pub trait Generate<const NCOLS: usize>: Display {
    // TODO: docstring
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS>;
}

pub struct UniformBox<const NCOLS: usize> {
    pub center: [FloatType; NCOLS],
    pub size: FloatType,
}

impl<const NCOLS: usize> Generate<NCOLS> for UniformBox<NCOLS> {
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS> {
        let mut rng = rng();
        FloatMatrixType::<NCOLS>::from_fn(n_pts, |_, col| {
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
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS> {
        let mut rng = rng();
        let normal_dist = Normal::new(0.0, 1.0).unwrap();
        let mut points =
            FloatMatrixType::<NCOLS>::from_fn(n_pts, |_, _| normal_dist.sample(&mut rng));

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

pub(crate) mod test {
    use crate::{config::test::TestDefault, generate::UniformBox, types::{FloatMatrixType, FloatType}};
    use nalgebra::{Const, RowVector};

    impl<const NCOLS: usize> TestDefault for UniformBox<NCOLS> {
        fn test_default() -> Self {
            UniformBox {
                center: [5.0; NCOLS],
                size: 5.0,
            }
        }
    }

    /// Dataset generator function commonly used in unit tests.
    ///
    /// Generate a dataset that has point_1 repeated n_1-times, followed by point_2 repeated n_2-times.
    #[allow(dead_code)]
    pub fn generate_2_point_dataset<const NCOLS: usize>(
        point_1: [FloatType; NCOLS],
        n_1: usize,
        point_2: [FloatType; NCOLS],
        n_2: usize,
    ) -> FloatMatrixType<NCOLS> {
        let mut rows = vec![RowVector::<FloatType, Const<NCOLS>, _>::from_row_slice(&point_1); n_1];
        rows.append(
            &mut vec![RowVector::<FloatType, Const<NCOLS>, _>::from_row_slice(&point_2); n_2],
        );
        FloatMatrixType::from_rows(&rows)
    }
}