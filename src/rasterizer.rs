use crate::types::{FloatMatrixType, FloatType, IndexType, IndexVectorType, RasterType};
use indexmap::IndexMap;
use nalgebra::{Const, Dyn, OMatrix, OVector};
use rapidhash::fast;

pub struct Rasterizer<const NCOLS: usize> {
    mapping: IndexVectorType,
    centroids: FloatMatrixType<NCOLS>,
    weights: IndexVectorType,
}

impl<const NCOLS: usize> Rasterizer<NCOLS> {
    pub fn new(input: &FloatMatrixType<NCOLS>, raster_res: FloatType) -> Self {
        assert!(input.nrows() <= IndexType::MAX as usize);
        assert!(raster_res > 0.0);

        let mut bin_map =
            IndexMap::<[RasterType; NCOLS], IndexType, fast::GlobalState>::with_hasher(
                fast::GlobalState::default(),
            );
        let mut mapping = IndexVectorType::zeros(input.nrows());

        // Bin the points
        let mut binned_float = input / raster_res;
        binned_float.iter_mut().for_each(|val| *val = val.floor());

        // NOTE: need to be transposed so that when accessing columns (rows in the original layout), the memory is
        // contiguous
        let binned = OMatrix::<RasterType, Dyn, Const<NCOLS>>::from_iterator(
            binned_float.nrows(),
            binned_float.iter().map(|&val| val as RasterType),
        )
        .transpose();

        // Iterate over the binned points and assign them to bins
        for (idx, bin) in binned.column_iter().enumerate() {
            // NOTE: NCOLS makes sure this will never panic
            let entry = bin_map.entry(bin.as_slice().try_into().unwrap());

            // Store the centroid ID in the mapping
            unsafe {
                *mapping.get_unchecked_mut(idx) = entry.index() as IndexType;
            }

            // Insert value into the mapping if new, and increase count
            let val = entry.or_insert_with(|| 0);
            *val += 1;
        }

        Self {
            mapping,
            // NOTE: if this would need to be super precise, the centroids would need to be shifted by the half of
            // raster_res in every dimension, because the "binned_float" coordinates represent the "lower left"
            // corners of each bin. However, this would result in a translation of the whole coordinate system,
            // which does not affect the distance calculations, so it can be omitted.
            centroids: FloatMatrixType::from_row_slice(
                &bin_map
                    .keys()
                    .flat_map(|key| key.iter().map(|&val| (val as FloatType) * raster_res))
                    .collect::<Vec<_>>(),
            ),
            weights: IndexVectorType::from_iterator(bin_map.len(), bin_map.values().copied()),
        }
    }

    pub fn rasterize(&self) -> (&FloatMatrixType<NCOLS>, &IndexVectorType) {
        (&self.centroids, &self.weights)
    }

    pub fn map_back(&self, cluster_ids: &IndexVectorType) -> IndexVectorType {
        assert!(self.centroids.nrows() == cluster_ids.nrows());

        let mut mapped_ids = OVector::<IndexType, Dyn>::zeros(self.mapping.nrows());

        for (id, &idx) in mapped_ids.iter_mut().zip(self.mapping.iter()) {
            unsafe {
                *id = *cluster_ids.get_unchecked(idx as usize);
            }
        }

        mapped_ids
    }
}

#[cfg(test)]
mod test {
    use crate::{
        generate::test::generate_2_point_dataset,
        rasterizer::Rasterizer,
        types::{FloatMatrixType, FloatType, IndexType, IndexVectorType},
    };
    use approx::assert_relative_eq;
    use nalgebra::{Dyn, OVector};
    use rstest::rstest;

    #[rstest]
    #[case([0.0, 0.0], 1.0, [0.0, 0.0])]
    #[case([-0.0, -0.0], 1.0, [0.0, 0.0])]
    #[case([1.0, 2.0], 1.0, [1.0, 2.0])]
    #[case([1.0, 2.0], 0.3, [0.9, 1.8])]
    #[case([-0.01, -0.01], 1.0, [-1.0, -1.0])]
    fn test_rasterizer_bins(
        #[case] input: [FloatType; 2],
        #[case] raster_res: FloatType,
        #[case] expected: [FloatType; 2],
    ) {
        assert_relative_eq!(
            Rasterizer::new(&FloatMatrixType::<2>::from_row_slice(&input), raster_res).centroids,
            &FloatMatrixType::from_row_slice(&expected),
            epsilon = FloatType::EPSILON
        );
    }

    #[rstest]
    #[case([0.0, 0.0], 1, [0.0, 0.0], 1, 1.0, &[0.0, 0.0], &[2])]
    #[case([0.0, 0.1], 1, [1.1, 1.1], 1, 1.0, &[0.0, 0.0, 1.0, 1.0], &[1, 1])]
    #[case([0.0, 0.1], 1, [1.1, 1.1], 10, 1.0, &[0.0, 0.0, 1.0, 1.0], &[1, 10])]
    #[case([0.0, 0.1], 10, [1.1, 1.1], 1, 1.0, &[0.0, 0.0, 1.0, 1.0], &[10, 1])]
    #[case([-0.1, -0.1], 10, [0.1, 0.1], 10, 1.0, &[-1.0, -1.0, 0.0, 0.0], &[10, 10])]
    fn test_rasterizer_e2e(
        #[case] point_1: [FloatType; 2],
        #[case] n_1: usize,
        #[case] point_2: [FloatType; 2],
        #[case] n_2: usize,
        #[case] raster_res: FloatType,
        #[case] expected_centroids: &[FloatType],
        #[case] expected_weights: &[IndexType],
    ) {
        let rasterizer = Rasterizer::new(
            &generate_2_point_dataset(point_1, n_1, point_2, n_2),
            raster_res,
        );
        let (centroids, weights) = rasterizer.rasterize();

        assert_relative_eq!(
            centroids,
            &FloatMatrixType::from_row_slice(&expected_centroids),
            epsilon = FloatType::EPSILON
        );
        assert_eq!(
            weights,
            &OVector::<IndexType, Dyn>::from_column_slice(&expected_weights)
        );
    }

    #[rstest]
    #[case(1, [0.0, 0.0], 1, &[0], &[0, 0])]
    #[case(1, [10.0, 0.0], 1, &[0, 1], &[0, 1])]
    #[case(5, [10.0, 0.0], 3, &[0, 1], &[0, 0, 0, 0, 0, 1, 1, 1])]
    fn test_rasterizer_mapping(
        #[case] n_1: usize,
        #[case] point_2: [FloatType; 2],
        #[case] n_2: usize,
        #[case] cluster_ids: &[IndexType],
        #[case] expected: &[IndexType],
    ) {
        let rasterizer = Rasterizer::new(
            &generate_2_point_dataset([0.0, 0.0], n_1, point_2, n_2),
            1.0,
        );

        let mapped_ids = rasterizer.map_back(&IndexVectorType::from_column_slice(cluster_ids));
        let expected_ids = IndexVectorType::from_column_slice(expected);

        assert_eq!(mapped_ids, expected_ids)
    }

    #[test]
    #[should_panic]
    fn test_rasterizer_invalid_res() {
        Rasterizer::new(&FloatMatrixType::<2>::zeros(1), 0.0);
    }

    #[test]
    #[should_panic]
    fn test_rasterizer_invalid_map_input() {
        let rasterizer = Rasterizer::new(&FloatMatrixType::<2>::zeros(1), 1.0);
        rasterizer.map_back(&IndexVectorType::zeros(2));
    }
}
