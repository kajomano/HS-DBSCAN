use crate::types::{FloatType, IndexType, MatrixType, MatrixViewType};
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
        proximities: OMatrix<IndexType, Dyn, Dyn>,
    },
}

impl Proximity {
    pub fn new<const NCOLS: usize>(input: &MatrixType<NCOLS>, config: &ProximityConfig) -> Self {
        let proximities = match config.norm {
            NormConfig::L1 => Self::pairwise_proximities(input, L1Norm, config.eps),
            NormConfig::L2 => Self::pairwise_proximities(input, L2Norm, config.eps),
            NormConfig::L2Squared => Self::pairwise_proximities(input, L2SquaredNorm, config.eps),
            NormConfig::Linf => Self::pairwise_proximities(input, LinfNorm, config.eps),
        };

        Self::MATRIX {
            config: *config,
            proximities,
        }
    }

    /// Query the proximity of a point. Both the queried point and the result are indices.
    pub fn query_proximity(quey_idx: IndexType) -> Vec<IndexType> {
        
    }

    /// Calculate the pairwise proximity between all input points. Returns an NxN complete proximity matrix, in which 0
    /// means outside of the proximity, and anything larger than 0 means inside the proximity.
    fn pairwise_proximities<const NCOLS: usize, N: Norm>(
        input: &MatrixType<NCOLS>,
        norm: N,
        eps: FloatType
    ) -> OMatrix<IndexType, Dyn, Dyn> {
        let mut buffer = MatrixType::zeros(input.nrows());
        let mut distances = OMatrix::<IndexType, Dyn, Dyn>::zeros(input.nrows(), input.nrows());

        let n = input.nrows();

        // Calculate pairwise distances. Only calculates the lower diagonal!
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
                let norms = norm.apply_rowise(&buffer_view);
                let proximities = norms.component_ <= eps

                // Store in distances
                distances_col_view.copy_from(&(norms ));
            });

        // Fill the upper triangle too for ease of later access.
        distances.fill_upper_triangle_with_lower_triangle();

        distances
    }
}
