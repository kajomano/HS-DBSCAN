use crate::types::{FloatType, IndexType, MatrixType};
use rand::{Rng, rng};

pub fn generate_uniform<const NCOLS: usize>(
    n_points: IndexType,
    extent: FloatType,
) -> MatrixType<NCOLS> {
    let mut rng = rng();
    MatrixType::<NCOLS>::from_fn(n_points as usize, |_, _| rng.random_range(-extent..extent))
}
