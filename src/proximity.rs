use crate::{
    config::NormConfig,
    types::{IndexType, MatrixType},
};
use eyre::{OptionExt, Result, ensure};
use nalgebra::{
    ClosedAddAssign, ClosedSubAssign, Const, Dyn, Matrix, OMatrix, OVector, Scalar, SimdValue,
    Storage, Vector, VectorView,
};
use num::{NumCast, Signed, Zero};
use std::{
    cmp::PartialOrd,
    ops::{Mul, SubAssign},
};

type VectorViewType<const NDIMS: usize, T, S> = Matrix<T, Const<NDIMS>, Const<1>, S>;

trait Norm {
    fn apply<
        const NDIMS: usize,
        T: Scalar
            + Zero
            + Signed
            + ClosedAddAssign
            + ClosedSubAssign
            + Mul<Output = T>
            + SimdValue<Element = T, SimdBool = bool>
            + PartialOrd
            + Copy,
        S: Storage<T, Const<NDIMS>, Const<1>>,
    >(
        &self,
        input: &VectorViewType<NDIMS, T, S>,
    ) -> T;
}

struct L1Norm;

impl Norm for L1Norm {
    fn apply<
        const NDIMS: usize,
        T: Scalar + Zero + Signed + ClosedAddAssign,
        S: Storage<T, Const<NDIMS>, Const<1>>,
    >(
        &self,
        input: &VectorViewType<NDIMS, T, S>,
    ) -> T {
        input.abs().sum()
    }
}

struct L2Norm;

impl Norm for L2Norm {
    fn apply<
        const NDIMS: usize,
        T: Scalar + Zero + Mul<Output = T> + Copy,
        S: Storage<T, Const<NDIMS>, Const<1>>,
    >(
        &self,
        input: &VectorViewType<NDIMS, T, S>,
    ) -> T {
        input.iter().fold(T::zero(), |acc, &v| acc + v * v)
    }
}

struct LinfNorm;

impl Norm for LinfNorm {
    fn apply<
        const NDIMS: usize,
        T: Scalar + Zero + Signed + SimdValue<Element = T, SimdBool = bool> + PartialOrd + Copy,
        S: Storage<T, Const<NDIMS>, Const<1>>,
    >(
        &self,
        input: &VectorViewType<NDIMS, T, S>,
    ) -> T {
        input.abs().max()
    }
}

pub trait Proximity {
    /// Returns a vector of weights where a weight > 0 means the point is within proximity of the
    /// query point. The query point is referenced by index.
    ///
    /// SAFETY: marked unsafe because a query_idx larger than the stored points can cause a panic.
    unsafe fn query<'a>(&'a self, query_idx: usize) -> VectorView<'a, IndexType, Dyn>;

    /// Returns the number of points stored.
    fn len(&self) -> usize;
}

/// RAII struct for handling proximity by precalculating every distance between point pairs.
pub struct MatrixProximity {
    proximities: OMatrix<IndexType, Dyn, Dyn>,
}

impl MatrixProximity {
    pub fn new<
        const NDIMS: usize,
        T: Scalar
            + Zero
            + Signed
            + ClosedAddAssign
            + ClosedSubAssign
            + Mul<Output = T>
            + SimdValue<Element = T, SimdBool = bool>
            + PartialOrd
            + Copy
            + NumCast,
        S: Storage<IndexType, Dyn>,
    >(
        input: &MatrixType<NDIMS, T>,
        weights: &Vector<IndexType, Dyn, S>,
        eps: f64,
        norm: &NormConfig,
    ) -> Result<Self> {
        let eps = T::from(eps).ok_or_eyre("Couldn't cast eps to T")?;

        ensure!(input.ncols() <= IndexType::MAX as usize);
        ensure!(input.ncols() == weights.nrows());
        ensure!(eps >= T::zero());

        let proximities = match norm {
            NormConfig::L1 => Self::pairwise_proximities(input, weights, eps, L1Norm),
            NormConfig::L2 => Self::pairwise_proximities(input, weights, eps * eps, L2Norm),
            NormConfig::Linf => Self::pairwise_proximities(input, weights, eps, LinfNorm),
        };

        Ok(Self { proximities })
    }

    /// Calculate the pairwise proximity between all input points.
    ///
    /// Returns an NxN complete proximity matrix, in which 0 means outside of the proximity, and anything larger than 0
    /// means inside the proximity. Larger than 0 values in the proximity matrix contain the weight of the point.
    fn pairwise_proximities<
        const NDIMS: usize,
        T: Scalar
            + Zero
            + Signed
            + ClosedAddAssign
            + ClosedSubAssign
            + Mul<Output = T>
            + SimdValue<Element = T, SimdBool = bool>
            + PartialOrd
            + Copy,
        N: Norm,
        S: Storage<IndexType, Dyn>,
    >(
        input: &MatrixType<NDIMS, T>,
        weights: &Vector<IndexType, Dyn, S>,
        eps: T,
        norm: N,
    ) -> OMatrix<IndexType, Dyn, Dyn> {
        let n = input.ncols();

        let mut buffer = MatrixType::<NDIMS, T>::zeros(n);
        let mut proximities = OMatrix::<IndexType, Dyn, Dyn>::zeros(n, n);

        // Calculate pairwise proximities. Only calculates the lower diagonal!
        for (mut proximities_col, (i, anchor)) in proximities
            .column_iter_mut()
            .zip(input.column_iter().enumerate())
        {
            // View for only working on the lower triangle
            let input_view = input.columns(i, n - i);
            let mut buffer_view = buffer.columns_mut(i, n - i);
            let mut proximities_col_view = proximities_col.rows_mut(i, n - i);

            // buffer = anchor.repeat()
            for mut col in buffer_view.column_iter_mut() {
                col.copy_from(&anchor)
            }

            // buffer = buffer - input
            buffer_view.sub_assign(input_view);

            // Calculate norm, threshold proximity
            // NOTE: even though this temp can be avoided, somehow this is faster
            let temp_proximities = OVector::<IndexType, Dyn>::from_iterator(
                buffer_view.ncols(),
                buffer_view
                    .column_iter()
                    .map(|col| if norm.apply(&col) <= eps { 1 } else { 0 }),
            );

            // Store in proximities col
            proximities_col_view.copy_from(&temp_proximities);
        }

        // Fill the upper triangle too
        proximities.fill_upper_triangle_with_lower_triangle();

        // Set weights
        for mut proximities_col in proximities.column_iter_mut() {
            proximities_col.component_mul_assign(weights)
        }

        proximities
    }
}

impl Proximity for MatrixProximity {
    unsafe fn query<'a>(&'a self, query_idx: usize) -> VectorView<'a, IndexType, Dyn> {
        self.proximities.column(query_idx)
    }

    fn len(&self) -> usize {
        self.proximities.nrows()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        config::{NormConfig, ProximityConfig, test::TestDefault},
        generate::test::generate_2_point_dataset,
        proximity::{L1Norm, L2Norm, LinfNorm, MatrixProximity, Norm},
        types::{FloatMatrixType, FloatType, IndexType},
    };
    use approx::assert_relative_eq;
    use nalgebra::{Dyn, OMatrix, OVector, Vector2};
    use rstest::rstest;

    #[rstest]
    #[case(L1Norm, [1.0, 2.0], 3.0)]
    #[case(L1Norm, [1.0, -2.0], 3.0)]
    #[case(L2Norm, [1.0, 2.0], 5.0)]
    #[case(L2Norm, [1.0, -2.0], 5.0)]
    #[case(LinfNorm, [1.0, 2.0], 2.0)]
    #[case(LinfNorm, [1.0, -2.0], 2.0)]
    fn test_norms<N: Norm>(
        #[case] norm: N,
        #[case] input: [FloatType; 2],
        #[case] expected: FloatType,
    ) {
        assert_relative_eq!(
            norm.apply(&Vector2::from_column_slice(&input)),
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
    #[case([0.0, 0.0], [2.0, 2.0], 1.0, NormConfig::Linf, 0)]
    #[case([-2.0, -2.0], [0.0, 0.0], 3.0, NormConfig::Linf, 1)]
    fn test_matrix_proximity_pairs(
        #[case] point_1: [FloatType; 2],
        #[case] point_2: [FloatType; 2],
        #[case] eps: f64,
        #[case] norm: NormConfig,
        #[case] expected: IndexType,
    ) {
        let prox = MatrixProximity::new(
            &generate_2_point_dataset(point_1, 1, point_2, 1),
            &OVector::<IndexType, Dyn>::repeat(2, 1),
            eps,
            &norm,
        )
        .unwrap();

        let expected =
            OMatrix::<IndexType, Dyn, Dyn>::from_column_slice(2, 2, &[1, expected, expected, 1]);

        assert_eq!(prox.proximities, expected)
    }

    #[rstest]
    #[case([0.0, 0.0], 1, [0.0, 0.0], 1, [1, 1, 1, 1])]
    #[case([0.0, 0.0], 0, [0.0, 0.0], 1, [0, 1, 0, 1])]
    #[case([0.0, 0.0], 0, [0.0, 0.0], 10, [0, 10, 0, 10])]
    #[case([0.0, 0.0], 1000, [10.0, 0.0], 10, [1000, 0, 0, 10])]
    fn test_matrix_proximity_weights(
        #[case] point_1: [FloatType; 2],
        #[case] weight_1: IndexType,
        #[case] point_2: [FloatType; 2],
        #[case] weight_2: IndexType,
        #[case] expected: [IndexType; 4],
    ) {
        let ProximityConfig { eps, norm } = ProximityConfig::test_default();
        let prox = MatrixProximity::new(
            &generate_2_point_dataset(point_1, 1, point_2, 1),
            &OVector::<IndexType, Dyn>::from_column_slice(&[weight_1, weight_2]),
            eps,
            &norm,
        )
        .unwrap();

        let expected = OMatrix::<IndexType, Dyn, Dyn>::from_column_slice(2, 2, &expected);

        assert_eq!(prox.proximities, expected)
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(10)]
    fn test_marix_proximity_shape(#[case] n: usize) {
        let ProximityConfig { eps, norm } = ProximityConfig::test_default();
        let prox = MatrixProximity::new(
            &FloatMatrixType::<2>::zeros(n),
            &OVector::<IndexType, Dyn>::repeat(n, 1),
            eps,
            &norm,
        )
        .unwrap();

        assert_eq!(prox.proximities.shape(), (n, n))
    }

    #[test]
    #[should_panic]
    fn test_matrix_proximity_invalid_input_rows() {
        let ProximityConfig { eps, norm } = ProximityConfig::test_default();
        MatrixProximity::new(
            &FloatMatrixType::<2>::identity(2),
            &OVector::<IndexType, Dyn>::repeat(3, 1),
            eps,
            &norm,
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_matrix_proximity_invalid_eps() {
        MatrixProximity::new(
            &FloatMatrixType::<2>::identity(2),
            &OVector::<IndexType, Dyn>::repeat(2, 1),
            -1.0,
            &NormConfig::test_default(),
        )
        .unwrap();
    }
}
