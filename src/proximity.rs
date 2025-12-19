use crate::types::{FloatType, MatrixType, MatrixViewType};
use nalgebra::{Const, Dyn, LpNorm, OMatrix, OVector, Storage, UniformNorm};
use std::ops::SubAssign;
use strum_macros::Display;

/// Different supported [norms](https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html).
#[derive(Clone, Copy, Debug, Default, Display)]
pub enum NormConfig {
    L1,
    #[default]
    L2,
    L2Squared,
    Linf,
}

trait Norm {
    fn apply_rowise<const NCOLS: usize, S: Storage<FloatType, Dyn, Const<NCOLS>>>(
        &self,
        input: &MatrixViewType<NCOLS, S>,
    ) -> OVector<FloatType, Dyn>;
}

struct L1Norm;

impl Norm for L1Norm {
    fn apply_rowise<const NCOLS: usize, S: Storage<FloatType, Dyn, Const<NCOLS>>>(
        &self,
        input: &MatrixViewType<NCOLS, S>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.apply_norm(&LpNorm(1))),
        )
    }
}

struct L2Norm;

impl Norm for L2Norm {
    fn apply_rowise<const NCOLS: usize, S: Storage<FloatType, Dyn, Const<NCOLS>>>(
        &self,
        input: &MatrixViewType<NCOLS, S>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.norm()),
        )
    }
}

struct L2SquaredNorm;

impl Norm for L2SquaredNorm {
    fn apply_rowise<const NCOLS: usize, S: Storage<FloatType, Dyn, Const<NCOLS>>>(
        &self,
        input: &MatrixViewType<NCOLS, S>,
    ) -> OVector<FloatType, Dyn> {
        OVector::<FloatType, Dyn>::from_iterator(
            input.nrows(),
            input.row_iter().map(|row| row.norm_squared()),
        )
    }
}

struct LinfNorm;

impl Norm for LinfNorm {
    fn apply_rowise<const NCOLS: usize, S: Storage<FloatType, Dyn, Const<NCOLS>>>(
        &self,
        input: &MatrixViewType<NCOLS, S>,
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

        let n = input.nrows();

        // Calculate pairwise distances between all input points. Only calculates the lower diagonal!
        distances
            .column_iter_mut()
            .zip(input.row_iter().enumerate())
            .for_each(|(mut distances_col, (i, anchor))| {
                // View for only working on the lower triangle
                let input_view = input.rows(i, n - i);
                let mut buffer_view = buffer.rows_mut(0, n - i);
                let mut distances_col_view = distances_col.rows_mut(i, n - i);

                // buffer = anchor.rep()
                for mut row in buffer_view.row_iter_mut() {
                    row.copy_from(&anchor)
                }

                // buffer = buffer - input
                buffer_view.sub_assign(input_view);

                // Calculate norm and store in distances column
                distances_col_view.copy_from(&norm.apply_rowise(&buffer_view));
            });

        // Fill the upper triangle too for ease of later access.
        distances.fill_upper_triangle_with_lower_triangle();

        distances
    }
}
