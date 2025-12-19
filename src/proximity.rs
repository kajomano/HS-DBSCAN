use crate::types::{FloatType, MatrixType};
use nalgebra::{Dyn, LpNorm, OMatrix, OVector, UniformNorm};
use std::ops::SubAssign;

/// Different supported [norms](https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html).
#[derive(Clone, Copy, Debug, Default)]
pub enum NormConfig {
    L1,
    #[default]
    L2,
    L2Squared,
    Linf,
}

trait Norm {
    fn apply_rowise<const NCOLS: usize>(
        &self,
        input: &MatrixType<NCOLS>,
    ) -> OVector<FloatType, Dyn>;
}

struct L1Norm;

impl Norm for L1Norm {
    fn apply_rowise<const NCOLS: usize>(
        &self,
        input: &MatrixType<NCOLS>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.apply_norm(&LpNorm(1))),
        )
    }
}

struct L2Norm;

impl Norm for L2Norm {
    fn apply_rowise<const NCOLS: usize>(
        &self,
        input: &MatrixType<NCOLS>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.norm()),
        )
    }
}

struct L2SquaredNorm;

impl Norm for L2SquaredNorm {
    fn apply_rowise<const NCOLS: usize>(
        &self,
        input: &MatrixType<NCOLS>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.norm_squared()),
        )
    }
}

struct LinfNorm;

impl Norm for LinfNorm {
    fn apply_rowise<const NCOLS: usize>(
        &self,
        input: &MatrixType<NCOLS>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.apply_norm(&UniformNorm)),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProximityConfig {
    eps: FloatType,
    norm: NormConfig,
}

impl Default for ProximityConfig {
    fn default() -> Self {
        Self::new(3.0, Default::default())
    }
}

impl ProximityConfig {
    pub fn new(eps: FloatType, norm: NormConfig) -> Self {
        Self {
            eps: match norm {
                NormConfig::L2Squared => eps * eps,
                _ => eps,
            },
            norm,
        }
    }

    pub fn eps(&self) -> FloatType {
        self.eps
    }

    pub fn norm(&self) -> NormConfig {
        self.norm
    }
}

// TODO: KD-tree
/// RAII struct for handling proximity
pub enum Proximity {
    MATRIX {
        config: ProximityConfig,
        distances: OMatrix<FloatType, Dyn, Dyn>,
    },
}

impl Proximity {
    pub fn new<const NCOLS: usize>(input: &MatrixType<NCOLS>, config: &ProximityConfig) -> Self {
        let distances = match config.norm {
            NormConfig::L1 => Self::pairwise_distances(input, L1Norm),
            NormConfig::L2 => Self::pairwise_distances(input, L2Norm),
            NormConfig::L2Squared => Self::pairwise_distances(input, L2SquaredNorm),
            NormConfig::Linf => Self::pairwise_distances(input, LinfNorm),
        };

        Self::MATRIX {
            config: *config,
            distances,
        }
    }

    fn pairwise_distances<const NCOLS: usize, N: Norm>(
        input: &MatrixType<NCOLS>,
        norm: N,
    ) -> OMatrix<FloatType, Dyn, Dyn> {
        let mut buffer = MatrixType::zeros(input.nrows());
        let mut distances = OMatrix::<FloatType, Dyn, Dyn>::zeros(input.nrows(), input.nrows());

        // Calculate pairwise distances between all input points
        distances.column_iter_mut().zip(input.row_iter()).for_each(
            |(mut distances_col, anchor)| {
                // buffer = anchor.rep()
                for mut row in buffer.row_iter_mut() {
                    row.copy_from(&anchor)
                }

                // buffer = buffer - input
                buffer.sub_assign(input);

                // Calculate norm and store in distances column
                distances_col.copy_from(&norm.apply_rowise(input));
            },
        );

        distances
    }
}
