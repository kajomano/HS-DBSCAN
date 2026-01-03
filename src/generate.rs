use crate::types::{FloatMatrixType, FloatType};
use nalgebra::stack;
use rand::{Rng, rng};
use rand_distr::{Distribution, Normal};
use std::fmt::{Display, Formatter, Result};

pub trait Generate<const NCOLS: usize>: Display {
    /// Generate a set of points according to the given distribution.
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS>;
}

/// Uniform distribution in the range `[center-size, center+size]`.
pub struct UniformBox<const NCOLS: usize> {
    pub center: [FloatType; NCOLS],
    pub size: FloatType,
}

impl<const NCOLS: usize> Generate<NCOLS> for UniformBox<NCOLS> {
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS> {
        assert!(self.size > 0.0);

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

/// Uniform distribution in the sphere given by `(center, radius)`.
pub struct UniformSphere<const NCOLS: usize> {
    pub center: [FloatType; NCOLS],
    pub radius: FloatType,
}

impl<const NCOLS: usize> Generate<NCOLS> for UniformSphere<NCOLS> {
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS> {
        assert!(self.radius > 0.0);

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

pub struct TwoUniformSpheres<const NCOLS: usize> {
    pub center: [FloatType; NCOLS],
    pub size: FloatType,
    pub offset: FloatType,
    pub radius: FloatType,
    pub noise_ratio: f64,
}

impl<const NCOLS: usize> Generate<NCOLS> for TwoUniformSpheres<NCOLS> {
    fn generate(&self, n_pts: usize) -> FloatMatrixType<NCOLS> {
        assert!(self.noise_ratio >= 0.0 && self.noise_ratio <= 1.0);

        let n_noise = ((n_pts as f64) * self.noise_ratio) as usize;
        let n_sphere = (n_pts - n_noise) / 2;

        let noise = UniformBox {
            center: self.center,
            size: self.size,
        };

        let sphere_1 = UniformSphere::<NCOLS> {
            center: self
                .center
                .iter()
                .map(|&val| val - self.offset)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            radius: self.radius,
        };

        let sphere_2 = UniformSphere::<NCOLS> {
            center: self
                .center
                .iter()
                .map(|&val| val + self.offset)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            radius: self.radius,
        };

        stack!(
            noise.generate(n_noise);
            sphere_1.generate(n_sphere);
            sphere_2.generate(n_sphere);
        )
    }
}

impl<const NCOLS: usize> Display for TwoUniformSpheres<NCOLS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "2uniformspheres")
    }
}

pub(crate) mod test {
    use crate::{
        config::test::TestDefault,
        generate::{TwoUniformSpheres, UniformBox, UniformSphere},
        types::{FloatMatrixType, FloatType},
    };
    use nalgebra::{Const, RowVector};

    impl<const NCOLS: usize> TestDefault for UniformBox<NCOLS> {
        fn test_default() -> Self {
            UniformBox {
                center: [0.0; NCOLS],
                size: 5.0,
            }
        }
    }

    impl<const NCOLS: usize> TestDefault for UniformSphere<NCOLS> {
        fn test_default() -> Self {
            UniformSphere {
                center: [0.0; NCOLS],
                radius: 0.1,
            }
        }
    }

    impl<const NCOLS: usize> TestDefault for TwoUniformSpheres<NCOLS> {
        fn test_default() -> Self {
            TwoUniformSpheres {
                center: [0.0; NCOLS],
                size: 5.0,
                offset: 2.0,
                radius: 1.0,
                noise_ratio: 0.2,
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
