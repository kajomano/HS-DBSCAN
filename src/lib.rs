use crate::{
    dbscan::Dbscan,
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
pub fn hs_dbscan<const NCOLS: usize>(input: &MatrixType<NCOLS>, config: &HsDbscanConfig) {
    // Create a proximity calculator
    let prox = MatrixProximity::new(input, &config.proximity);

    // Create the dbscan object
    let mut dbscan = Dbscan::new(input, config.min_pts);
    let mut cluster_id: IndexType = 1;

    for idx in 0..input.nrows() {
        if dbscan.expand_cluster(&prox, idx, cluster_id) {
            cluster_id += 1;
        }
    }
}
