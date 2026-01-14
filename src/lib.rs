use crate::{
    config::HsDbscanConfig,
    dbscan::dbscan,
    proximity::MatrixProximity,
    rasterizer::Rasterizer,
    types::{FloatMatrixType, IndexType, IndexVectorType},
};
use eyre::Result;
use nalgebra::{Dyn, OVector};

pub mod config;
pub mod dbscan;
pub mod generate;
pub mod proximity;
pub mod rasterizer;
pub mod types;

/// High-Speed DBSCAN algorithm.
///
/// Implements an approximate [DBSCAN](resources/dbscan.pdf) algorithm, with focus on execution speed for smaller
/// datasets (~100-10k points).
///
/// The approximate nature of the algorithm comes from a quantization/binning/rasterization step before the clustering,
/// where each input point is assigned to a rectangular bin. The clustering then runs on the centroids of the bins,
/// which potentially drastically reduces the effective number of input points. To retain the density-oriented nature of
/// DBSCAN, the centroids are weighted by the number of datapoints they represent, and the modified DBSCAN algorithm
/// respects these weights when estimating density.
pub fn hs_dbscan<const NDIMS: usize>(
    input: &FloatMatrixType<NDIMS>,
    config: &HsDbscanConfig,
) -> Result<IndexVectorType> {
    if let Some(raster_res) = config.raster_res {
        // Rasterize the input
        let rasterizer = Rasterizer::new(input, raster_res)?;
        let (input, weights) = rasterizer.rasterize();

        // Create a proximity calculator
        let prox = MatrixProximity::new(input, weights, &config.proximity)?;

        // Run dbscan and map the cluster IDs back to the original input points
        rasterizer.map_back(&dbscan(&prox, config.min_pts))
    } else {
        // Create a proximity calculator with mock weights
        let prox = MatrixProximity::new(
            input,
            &OVector::<IndexType, Dyn>::repeat(input.ncols(), 1),
            &config.proximity,
        )?;

        // Run dbscan
        Ok(dbscan(&prox, config.min_pts))
    }
}
