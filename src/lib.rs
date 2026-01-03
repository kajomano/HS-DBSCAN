use crate::{
    config::HsDbscanConfig,
    dbscan::dbscan,
    proximity::MatrixProximity,
    rasterizer::Rasterizer,
    types::{FloatMatrixType, IndexType, IndexVectorType},
};
use nalgebra::{Dyn, OVector};

pub mod config;
pub mod dbscan;
pub mod generate;
pub mod proximity;
pub mod rasterizer;
pub mod types;

// TODO: docstring
pub fn hs_dbscan<const NCOLS: usize>(
    input: &FloatMatrixType<NCOLS>,
    config: &HsDbscanConfig,
) -> IndexVectorType {
    if let Some(raster_res) = config.raster_res {
        // Rasterize the input
        let rasterizer = Rasterizer::new(input, raster_res);
        let (input, weights) = rasterizer.rasterize();

        // Create a proximity calculator
        let prox = MatrixProximity::new(input, &weights, &config.proximity);

        // Run dbscan and map the cluster IDs back to the original input points
        rasterizer.map_back(&dbscan(&prox, config.min_pts))
    } else {
        // Create a proximity calculator with mock weights
        let prox = MatrixProximity::new(
            input,
            &OVector::<IndexType, Dyn>::repeat(input.nrows(), 1),
            &config.proximity,
        );

        // Run dbscan
        dbscan(&prox, config.min_pts)
    }
}
