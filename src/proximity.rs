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
        Self {
            eps: 3.0,
            norm: Default::default(),
        }
    }
}

pub trait Proximity {
    /// Returns a vector of weights where a weight > 0 means the point is within proximity of the
    /// query point. The query point is referenced by index.
    fn query<'a>(&'a self, query_idx: usize) -> VectorView<'a, IndexType, Dyn>;
}

// TODO: KD-tree
/// RAII struct for handling proximity by precalculating every distance between point pairs.
pub struct MatrixProximity {
    proximities: OMatrix<IndexType, Dyn, Dyn>,
}

impl MatrixProximity {
    pub fn new<const NCOLS: usize>(input: &MatrixType<NCOLS>, config: &ProximityConfig) -> Self {
        assert!(config.eps >= 0.0);

        let proximities = match config.norm {
            NormConfig::L1 => Self::pairwise_proximities(input, L1Norm, config.eps),
            NormConfig::L2 => Self::pairwise_proximities(input, L2Norm, config.eps),
            NormConfig::L2Squared => {
                Self::pairwise_proximities(input, L2SquaredNorm, config.eps * config.eps)
            }
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
                        .map(|row| if norm.apply(&row) <= eps { 1 } else { 0 }),
                );

                // Store in proximities cole
                proximities_col_view.copy_from(&temp_proximities);
            });

        // Fill the upper triangle too for ease of later access.
        proximities.fill_upper_triangle_with_lower_triangle();

        proximities
    }
}

impl Proximity for MatrixProximity {
    fn query<'a>(&'a self, query_idx: usize) -> VectorView<'a, IndexType, Dyn> {
        self.proximities.column(query_idx)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        proximity::{
            L1Norm, L2Norm, L2SquaredNorm, LinfNorm, MatrixProximity, Norm, NormConfig,
            ProximityConfig,
        },
        types::{FloatType, IndexType, MatrixType},
    };
    use approx::assert_relative_eq;
    use nalgebra::{Dyn, OMatrix, RowVector2};
    use rstest::rstest;

    fn f(num: FloatType) -> FloatType {
        num
    }

    #[rstest]
    #[case(L1Norm, [1.0, 2.0], 3.0)]
    #[case(L1Norm, [1.0, -2.0], 3.0)]
    #[case(L2Norm, [1.0, 2.0], f(5.0).sqrt())]
    #[case(L2SquaredNorm, [1.0, 2.0], 5.0)]
    #[case(LinfNorm, [1.0, 2.0], 2.0)]
    #[case(LinfNorm, [1.0, -2.0], 2.0)]
    fn test_norms<N: Norm>(
        #[case] norm: N,
        #[case] input: [FloatType; 2],
        #[case] expected: FloatType,
    ) {
        assert_relative_eq!(
            norm.apply(&RowVector2::from_row_slice(&input)),
            expected,
            epsilon = FloatType::EPSILON
        );
    }

    #[rstest]
    #[case([0.0, 0.0], [0.0, 0.0], 3.0, NormConfig::L1, 1)]
    #[case([0.0, 0.0], [0.0, 0.0], 0.0, NormConfig::L1, 1)]
    #[case([0.0, 0.0], [2.0, 2.0], 3.0, NormConfig::L1, 0)]
    #[case([-2.0, -2.0], [0.0, 0.0], 5.0, NormConfig::L1, 1)]
    #[case([0.0, 0.0], [2.0, 2.0], 2.0, NormConfig::L2, 0)]
    #[case([-2.0, -2.0], [0.0, 0.0], 3.0, NormConfig::L2, 1)]
    #[case([0.0, 0.0], [2.0, 2.0], 2.0, NormConfig::L2Squared, 0)]
    #[case([-2.0, -2.0], [0.0, 0.0], 3.0, NormConfig::L2Squared, 1)]
    #[case([0.0, 0.0], [2.0, 2.0], 1.0, NormConfig::Linf, 0)]
    #[case([-2.0, -2.0], [0.0, 0.0], 3.0, NormConfig::Linf, 1)]
    fn test_matrix_proximity_pairs(
        #[case] point_1: [FloatType; 2],
        #[case] point_2: [FloatType; 2],
        #[case] eps: FloatType,
        #[case] norm: NormConfig,
        #[case] expected: IndexType,
    ) {
        let input = MatrixType::from_rows(&[
            RowVector2::from_row_slice(&point_1),
            RowVector2::from_row_slice(&point_2),
        ]);

        let expected =
            OMatrix::<IndexType, Dyn, Dyn>::from_row_slice(2, 2, &[1, expected, expected, 1]);

        let prox = MatrixProximity::new(&input, &ProximityConfig { eps, norm });

        assert_eq!(prox.proximities, expected)
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(10)]
    fn test_marix_proximity_shape(#[case] n: usize) {
        let prox = MatrixProximity::new(&MatrixType::<2>::zeros(n), &Default::default());
        assert_eq!(prox.proximities.shape(), (n, n))
    }

    #[test]
    #[should_panic]
    fn test_matrix_proximity_invalid_eps() {
        let input = MatrixType::<2>::identity(2);

        MatrixProximity::new(
            &input,
            &ProximityConfig {
                eps: -1.0,
                norm: NormConfig::L1,
            },
        );
    }
}
