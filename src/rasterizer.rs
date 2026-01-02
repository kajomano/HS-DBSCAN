use crate::types::{FloatType, IndexType, MatrixType, RasterType};
use indexmap::IndexMap;
use nalgebra::{Dyn, OVector};
use rapidhash::fast::GlobalState;

pub struct Rasterizer<const NCOLS: usize> {
    mapping: OVector<IndexType, Dyn>,
    centroids: MatrixType<NCOLS>,
    weights: OVector<IndexType, Dyn>,
}

impl<const NCOLS: usize> Rasterizer<NCOLS> {
    pub fn new(input: &MatrixType<NCOLS>, raster_res: FloatType) -> Self {
        assert!(input.nrows() <= IndexType::MAX as usize);
        assert!(raster_res > 0.0);

        let mut bin_map =
            IndexMap::<Vec<RasterType>, IndexType, GlobalState>::with_hasher(GlobalState::new());
        // // NOTE: centroids could be re-calculated from the bin_map keys
        // let mut centroids = Vec::<&Matrix<FloatType, Const<1>, Const<NCOLS>, _>>::new();
        let mut mapping = OVector::<IndexType, Dyn>::zeros(input.nrows());

        // Bin the points
        let mut binned = input / raster_res;
        binned.iter_mut().for_each(|val| *val = val.floor());

        // Iterate over the binned points and assign them to bins
        for (idx, bin) in binned.row_iter().enumerate() {
            let entry = bin_map.entry(bin.iter().map(|&val| val as RasterType).collect());

            // Store the centroid ID in the mapping
            unsafe {
                *mapping.get_unchecked_mut(idx) = entry.index() as IndexType;
            }

            // Insert value if new, and increase count
            let val = entry.or_insert(0);
            *val += 1;

            // // If new entry, store the centroid
            // if new_entry {
            //     centroids.push(&row);
            // }
        }

        // Create the centroids matrix and weights vector
        let centroids = MatrixType::<NCOLS>::from_row_slice(
            &bin_map
                .keys()
                .flat_map(|key| key.iter().map(|&val| (val as FloatType) * raster_res))
                .collect::<Vec<_>>(),
        );

        let weights = OVector::<IndexType, Dyn>::from_iterator(
            bin_map.len(),
            bin_map.values().map(|&val| val),
        );

        Self {
            mapping,
            centroids,
            weights,
        }
    }

    pub fn rasterize(&self) -> (&MatrixType<NCOLS>, &OVector<IndexType, Dyn>) {
        (&self.centroids, &self.weights)
    }

    pub fn map_back(&self, cluster_ids: &OVector<IndexType, Dyn>) -> OVector<IndexType, Dyn> {
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
        rasterizer::Rasterizer,
        types::{FloatType, IndexType, MatrixType},
    };
    use approx::assert_relative_eq;
    use nalgebra::{Dyn, OVector, RowVector2};
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
            Rasterizer::new(&MatrixType::<2>::from_row_slice(&input), raster_res).centroids,
            &MatrixType::from_row_slice(&expected),
            epsilon = FloatType::EPSILON
        );
    }

    #[rstest]
    #[case([0.0, 0.0], 1, [0.0, 0.0], 1, 1.0, vec![0.0, 0.0], vec![2])]
    #[case([0.0, 0.1], 1, [1.1, 1.1], 1, 1.0, vec![0.0, 0.0, 1.0, 1.0], vec![1, 1])]
    #[case([0.0, 0.1], 1, [1.1, 1.1], 10, 1.0, vec![0.0, 0.0, 1.0, 1.0], vec![1, 10])]
    #[case([0.0, 0.1], 10, [1.1, 1.1], 1, 1.0, vec![0.0, 0.0, 1.0, 1.0], vec![10, 1])]
    #[case([-0.1, -0.1], 10, [0.1, 0.1], 10, 1.0, vec![-1.0, -1.0, 0.0, 0.0], vec![10, 10])]
    fn test_rasterizer_e2e(
        #[case] point_1: [FloatType; 2],
        #[case] n_1: usize,
        #[case] point_2: [FloatType; 2],
        #[case] n_2: usize,
        #[case] raster_res: FloatType,
        #[case] expected_centroids: Vec<FloatType>,
        #[case] expected_weights: Vec<IndexType>,
    ) {
        let mut rows = vec![RowVector2::from_row_slice(&point_1); n_1];
        rows.append(&mut vec![RowVector2::from_row_slice(&point_2); n_2]);
        let input = MatrixType::from_rows(&rows);

        let rasterizer = Rasterizer::new(&input, raster_res);
        let (centroids, weights) = rasterizer.rasterize();

        assert_relative_eq!(
            centroids,
            &MatrixType::from_row_slice(&expected_centroids),
            epsilon = FloatType::EPSILON
        );
        assert_eq!(
            weights,
            &OVector::<IndexType, Dyn>::from_column_slice(&expected_weights)
        );
    }

    #[test]
    #[should_panic]
    fn test_rasterizer_invalid_res() {
        Rasterizer::new(&MatrixType::<2>::zeros(1), 0.0);
    }
}
