use crate::{
    config::HsDbscanConfig,
    dbscan::dbscan,
    proximity::MatrixProximity,
    types::{IndexType, MatrixType},
};
use nalgebra::{Dyn, OVector};

pub mod config;
pub mod dbscan;
pub mod generate;
pub mod proximity;
pub mod rasterizer;
pub mod types;

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
