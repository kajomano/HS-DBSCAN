use crate::types::{FloatMatrixType, FloatType};
use eyre::{Result, ensure};
use nalgebra::stack;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rand_distr::{Distribution, Normal};
use std::fmt::{Display, Formatter};

pub trait Generate<const NDIMS: usize>: Display {
    /// Generate a set of points according to the given distribution.
    fn generate(&self, n_pts: usize) -> Result<FloatMatrixType<NDIMS>>;
}

/// Uniform distribution in the range `[center-size, center+size]`.
pub struct UniformBox<const NDIMS: usize> {
    pub center: [FloatType; NDIMS],
    pub size: FloatType,
}

impl<const NDIMS: usize> Generate<NDIMS> for UniformBox<NDIMS> {
    fn generate(&self, n_pts: usize) -> Result<FloatMatrixType<NDIMS>> {
        ensure!(self.size > 0.0);

        let mut rng = StdRng::from_seed([1; 32]);

        Ok(FloatMatrixType::<NDIMS>::from_fn(n_pts, |dim, _| {
            rng.random_range((self.center[dim] - (self.size))..(self.center[dim] + (self.size)))
        }))
    }
}

impl<const NDIMS: usize> Display for UniformBox<NDIMS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "uniformbox")
    }
}

/// Uniform distribution in the sphere given by `(center, radius)`.
pub struct UniformSphere<const NDIMS: usize> {
    pub center: [FloatType; NDIMS],
    pub radius: FloatType,
}

impl<const NDIMS: usize> Generate<NDIMS> for UniformSphere<NDIMS> {
    fn generate(&self, n_pts: usize) -> Result<FloatMatrixType<NDIMS>> {
        ensure!(self.radius > 0.0);

        let mut rng = StdRng::from_seed([2; 32]);

        // SAFETY: because of the hardcoded std_dev, this will never panic
        let normal_dist = Normal::new(0.0, 1.0).unwrap();
        let mut points =
            FloatMatrixType::<NDIMS>::from_fn(n_pts, |_, _| normal_dist.sample(&mut rng));

        let center = FloatMatrixType::<NDIMS>::from_column_slice(&self.center);

        for mut point in points.column_iter_mut() {
            let mut u: FloatType = rng.random_range(0.0..1.0);
            u = u.powf(1.0 / (NDIMS as FloatType)) * self.radius;
            point *= u / point.norm();
            point += &center;
        }

        Ok(points)
    }
}

impl<const NDIMS: usize> Display for UniformSphere<NDIMS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "uniformsphere")
    }
}

pub struct TwoUniformSpheres<const NDIMS: usize> {
    pub center: [FloatType; NDIMS],
    pub size: FloatType,
    pub offset: FloatType,
    pub radius: FloatType,
    pub noise_ratio: f64,
}

impl<const NDIMS: usize> Generate<NDIMS> for TwoUniformSpheres<NDIMS> {
    fn generate(&self, n_pts: usize) -> Result<FloatMatrixType<NDIMS>> {
        ensure!(self.noise_ratio >= 0.0 && self.noise_ratio <= 1.0);

        let n_noise = ((n_pts as f64) * self.noise_ratio) as usize;
        let n_sphere = (n_pts - n_noise) / 2;

        let noise = UniformBox {
            center: self.center,
            size: self.size,
        };

        let sphere_1 = UniformSphere::<NDIMS> {
            // SAFETY: NDIMS makes sure this will never panic
            center: self
                .center
                .iter()
                .map(|&val| val - self.offset)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            radius: self.radius,
        };

        let sphere_2 = UniformSphere::<NDIMS> {
            // SAFETY: NDIMS makes sure this will never panic
            center: self
                .center
                .iter()
                .map(|&val| val + self.offset)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            radius: self.radius,
        };

        Ok(
            stack!(noise.generate(n_noise)?, sphere_1.generate(n_sphere)?, sphere_2.generate(n_sphere)?;),
        )
    }
}

impl<const NDIMS: usize> Display for TwoUniformSpheres<NDIMS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "2uniformspheres")
    }
}

pub(crate) mod test {
    use crate::{
        config::test::TestDefault,
        generate::{TwoUniformSpheres, UniformBox, UniformSphere},
        types::{FloatMatrixType, FloatType},
    };
    use nalgebra::{Const, Vector};

    impl<const NDIMS: usize> TestDefault for UniformBox<NDIMS> {
        fn test_default() -> Self {
            UniformBox {
                center: [5.0; NDIMS],
                size: 5.0,
            }
        }
    }

    impl<const NDIMS: usize> TestDefault for UniformSphere<NDIMS> {
        fn test_default() -> Self {
            UniformSphere {
                center: [5.0; NDIMS],
                radius: 0.1,
            }
        }
    }

    impl<const NDIMS: usize> TestDefault for TwoUniformSpheres<NDIMS> {
        fn test_default() -> Self {
            TwoUniformSpheres {
                center: [5.0; NDIMS],
                size: 5.0,
                offset: 2.5,
                radius: 2.0,
                noise_ratio: 0.2,
            }
        }
    }

    /// Dataset generator function commonly used in unit tests.
    ///
    /// Generate a dataset that has point_1 repeated n_1-times, followed by point_2 repeated n_2-times.
    #[allow(dead_code)]
    pub fn generate_2_point_dataset<const NDIMS: usize>(
        point_1: [FloatType; NDIMS],
        n_1: usize,
        point_2: [FloatType; NDIMS],
        n_2: usize,
    ) -> FloatMatrixType<NDIMS> {
        let mut cols = vec![Vector::<FloatType, Const<NDIMS>, _>::from_column_slice(&point_1); n_1];
        cols.append(
            &mut vec![Vector::<FloatType, Const<NDIMS>, _>::from_column_slice(&point_2); n_2],
        );
        FloatMatrixType::from_columns(&cols)
    }
}
