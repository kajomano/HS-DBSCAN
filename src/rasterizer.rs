use crate::types::{FloatMatrixType, FloatType, IndexType, IndexVectorType, RasterType};
use eyre::{OptionExt, Result, ensure};
use indexmap::IndexMap;
use nalgebra::{Const, Dyn, OMatrix, OVector};
use num::{NumCast, Zero};
use rapidhash::fast;

pub struct Rasterizer<const NDIMS: usize> {
    mapping: IndexVectorType,
    centroids: FloatMatrixType<NDIMS>,
    weights: IndexVectorType,
}

impl<const NDIMS: usize> Rasterizer<NDIMS> {
    pub fn new(input: &FloatMatrixType<NDIMS>, raster_res: f64) -> Result<Self> {
        let raster_res = <FloatType as NumCast>::from(raster_res)
            .ok_or_eyre("Couldn't cast raster_res to FloatType")?;

        ensure!(input.ncols() <= IndexType::MAX as usize);
        ensure!(raster_res > <FloatType as Zero>::zero());

        let mut bin_map =
            IndexMap::<[RasterType; NDIMS], IndexType, fast::GlobalState>::with_hasher(
                fast::GlobalState::default(),
            );
        let mut mapping = IndexVectorType::zeros(input.ncols());

        // Bin the points
        let binned = OMatrix::<RasterType, Const<NDIMS>, Dyn>::from_iterator(
            input.ncols(),
            (input / raster_res)
                .iter()
                .map(|val| val.floor() as RasterType),
        );

        // Iterate over the binned points and assign them to bins
        for (idx, bin) in binned.column_iter().enumerate() {
            // SAFETY: NDIMS makes sure this will never panic
            let entry = bin_map.entry(bin.as_slice().try_into().unwrap());

            // Store the centroid ID in the mapping
            unsafe {
                *mapping.get_unchecked_mut(idx) = entry.index() as IndexType;
            }

            // Insert value into the mapping if new, and increase count
            let val = entry.or_insert(0);
            *val += 1;
        }

        Ok(Self {
            mapping,
            // NOTE: if this would need to be super precise, the centroids would need to be shifted by the half of
            // raster_res in every dimension, because the "binned" coordinates represent the "lower left"
            // corners of each bin. However, this would result in a translation of the whole coordinate system,
            // which does not affect the distance calculations, so it can be omitted.
            centroids: FloatMatrixType::from_iterator(
                bin_map.len(),
                bin_map
                    .keys()
                    .flat_map(|key| key.iter().map(|&val| (val as FloatType) * raster_res)),
            ),
            weights: IndexVectorType::from_iterator(bin_map.len(), bin_map.values().copied()),
        })
    }

    pub fn rasterize(&self) -> (&FloatMatrixType<NDIMS>, &IndexVectorType) {
        (&self.centroids, &self.weights)
    }

    pub fn map_back(&self, cluster_ids: &IndexVectorType) -> Result<IndexVectorType> {
        ensure!(self.centroids.ncols() == cluster_ids.nrows());

        let mut mapped_ids = OVector::<IndexType, Dyn>::zeros(self.mapping.nrows());

        for (id, &idx) in mapped_ids.iter_mut().zip(self.mapping.iter()) {
            unsafe {
                *id = *cluster_ids.get_unchecked(idx as usize);
            }
        }

        Ok(mapped_ids)
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
        #[case] raster_res: f64,
        #[case] expected: [FloatType; 2],
    ) {
        assert_relative_eq!(
            Rasterizer::new(&FloatMatrixType::<2>::from_column_slice(&input), raster_res)
                .unwrap()
                .centroids,
            &FloatMatrixType::from_column_slice(&expected),
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
        #[case] raster_res: f64,
        #[case] expected_centroids: &[FloatType],
        #[case] expected_weights: &[IndexType],
    ) {
        let rasterizer = Rasterizer::new(
            &generate_2_point_dataset(point_1, n_1, point_2, n_2),
            raster_res,
        )
        .unwrap();
        let (centroids, weights) = rasterizer.rasterize();

        assert_relative_eq!(
            centroids,
            &FloatMatrixType::from_column_slice(&expected_centroids),
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
        )
        .unwrap();

        let mapped_ids = rasterizer
            .map_back(&IndexVectorType::from_column_slice(cluster_ids))
            .unwrap();
        let expected_ids = IndexVectorType::from_column_slice(expected);

        assert_eq!(mapped_ids, expected_ids)
    }

    #[test]
    #[should_panic]
    fn test_rasterizer_invalid_res() {
        Rasterizer::new(&FloatMatrixType::<2>::zeros(1), 0.0).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_rasterizer_invalid_map_input() {
        let rasterizer = Rasterizer::new(&FloatMatrixType::<2>::zeros(1), 1.0).unwrap();
        rasterizer.map_back(&IndexVectorType::zeros(2)).unwrap();
    }
}
