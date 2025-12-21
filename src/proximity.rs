use crate::types::{FloatType, IndexType, MatrixType};
use nalgebra::{Const, Dyn, LpNorm, Matrix, OMatrix, OVector, Storage, UniformNorm, VectorView};
use std::ops::SubAssign;
use strum_macros::Display;

// NOTE: the bound on S is not enforced through type aliases, so add it as a bound on the function too!
#[allow(type_alias_bounds)]
type RowVectorViewType<const NCOLS: usize, S: Storage<FloatType, Const<1>, Const<NCOLS>>> =
    Matrix<FloatType, Const<1>, Const<NCOLS>, S>;

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
    fn apply<const NCOLS: usize, S: Storage<FloatType, Const<1>, Const<NCOLS>>>(
        &self,
        input: &RowVectorViewType<NCOLS, S>,
    ) -> FloatType;
}

struct L1Norm;

impl Norm for L1Norm {
    fn apply<const NCOLS: usize, S: Storage<FloatType, Const<1>, Const<NCOLS>>>(
        &self,
        input: &RowVectorViewType<NCOLS, S>,
    ) -> FloatType {
        input.apply_norm(&LpNorm(1))
    }
}

struct L2Norm;

impl Norm for L2Norm {
    fn apply<const NCOLS: usize, S: Storage<FloatType, Const<1>, Const<NCOLS>>>(
        &self,
        input: &RowVectorViewType<NCOLS, S>,
    ) -> FloatType {
        input.norm()
    }
}

struct L2SquaredNorm;

impl Norm for L2SquaredNorm {
    fn apply<const NCOLS: usize, S: Storage<FloatType, Const<1>, Const<NCOLS>>>(
        &self,
        input: &RowVectorViewType<NCOLS, S>,
    ) -> FloatType {
        input.norm_squared()
    }
}

struct LinfNorm;

impl Norm for LinfNorm {
    fn apply<const NCOLS: usize, S: Storage<FloatType, Const<1>, Const<NCOLS>>>(
        &self,
        input: &RowVectorViewType<NCOLS, S>,
    ) -> FloatType {
        input.apply_norm(&UniformNorm)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProximityConfig {
    pub eps: FloatType,
    pub norm: NormConfig,
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

pub trait Proximity {
    /// Returns a vector of weights where a weight > 0 means the point is within proximity of the
    /// query point. The query point is referenced by index.
    fn query<'a>(&'a self, query_idx: IndexType) -> VectorView<'a, IndexType, Dyn>;
}

// TODO: KD-tree
/// RAII struct for handling proximity by precalculating every distance between point pairs.
pub struct MatrixProximity {
    proximities: OMatrix<IndexType, Dyn, Dyn>,
}

impl MatrixProximity {
    pub fn new<const NCOLS: usize>(input: &MatrixType<NCOLS>, config: &ProximityConfig) -> Self {
        let proximities = match config.norm {
            NormConfig::L1 => Self::pairwise_proximities(input, L1Norm, config.eps),
            NormConfig::L2 => Self::pairwise_proximities(input, L2Norm, config.eps),
            NormConfig::L2Squared => Self::pairwise_proximities(input, L2SquaredNorm, config.eps),
            NormConfig::Linf => Self::pairwise_proximities(input, LinfNorm, config.eps),
        };

        Self { proximities }
    }

    /// Calculate the pairwise proximity between all input points. Returns an NxN complete proximity matrix, in which 0
    /// means outside of the proximity, and anything larger than 0 means inside the proximity.
    fn pairwise_proximities<const NCOLS: usize, N: Norm>(
        input: &MatrixType<NCOLS>,
        norm: N,
        eps: FloatType,
    ) -> OMatrix<IndexType, Dyn, Dyn> {
        let mut buffer = MatrixType::zeros(input.nrows());
        let mut proximities = OMatrix::<IndexType, Dyn, Dyn>::zeros(input.nrows(), input.nrows());

        let n = input.nrows();

        // Calculate pairwise proximities. Only calculates the lower diagonal!
        proximities
            .column_iter_mut()
            .zip(input.row_iter().enumerate())
            .for_each(|(mut proximities_col, (i, anchor))| {
                // View for only working on the lower triangle
                let input_view = input.rows(i, n - i);
                let mut buffer_view = buffer.rows_mut(0, n - i);
                let mut proximities_col_view = proximities_col.rows_mut(i, n - i);

                // buffer = anchor.rep()
                for mut row in buffer_view.row_iter_mut() {
                    row.copy_from(&anchor)
                }

                // buffer = buffer - input
                buffer_view.sub_assign(input_view);

                // TODO: handle weights
                // Calculate norm, threshold proximity
                // NOTE: even though this temp can be avoided, somehow this is faster
                let temp_proximities = OVector::<IndexType, Dyn>::from_iterator(
                    buffer_view.nrows(),
                    buffer_view
                        .row_iter()
                        .map(|row| if norm.apply(&row) < eps { 1 } else { 0 }),
                );

                // Store in proximities cole
                proximities_col_view.copy_from(&temp_proximities);

                // // TODO: Benchmark this on the othe hardware too
                // // Calculate norm, threshold proximity and store the effective weight in proximities column
                // proximities_col_view
                //     .iter_mut()
                //     .zip(buffer_view.row_iter())
                //     .for_each(|(proximity, buffer_row)| {
                //         if norm.apply(&buffer_row) < eps {
                //             *proximity = 1;
                //         }
                //     });
            });

        // Fill the upper triangle too for ease of later access.
        proximities.fill_upper_triangle_with_lower_triangle();

        proximities
    }
}

impl Proximity for MatrixProximity {
    fn query<'a>(&'a self, query_idx: IndexType) -> VectorView<'a, IndexType, Dyn> {
        self.proximities.column(query_idx as usize)
    }
}
