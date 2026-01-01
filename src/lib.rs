use nalgebra::{Dyn, OVector};

use crate::{
    dbscan::dbscan,
    proximity::{MatrixProximity, ProximityConfig},
    types::{FloatType, IndexType, MatrixType},
};

pub mod dbscan;
pub mod generate;
pub mod proximity;
pub mod types;

#[derive(Clone, Copy, Debug)]
pub struct HsDbscanConfig {
    pub raster_res: FloatType,
    pub proximity: ProximityConfig,
    pub min_pts: IndexType,
}

impl Default for HsDbscanConfig {
    fn default() -> Self {
        Self {
            raster_res: 1.0,
            proximity: Default::default(),
            min_pts: 10,
        }
    }
}

// TODO: docstring
// TODO: return
pub fn hs_dbscan<const NCOLS: usize>(input: &MatrixType<NCOLS>, config: &HsDbscanConfig) {
    // Create mock weights
    let weights = OVector::<IndexType, Dyn>::repeat(input.nrows(), 1);

    // Create a proximity calculator
    let prox = MatrixProximity::new(input, &weights, &config.proximity);

    // Run dbscan
    dbscan(&prox, config.min_pts);
}
