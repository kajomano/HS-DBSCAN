use crate::{
    proximity::{Proximity, ProximityConfig},
    types::{FloatType, MatrixType},
};

#[derive(Clone, Copy, Debug)]
pub struct HsDbscan<const NCOLS: usize> {
    pub raster_res: FloatType,
    pub proximity: ProximityConfig,
    pub min_pts: usize,
}

impl<const NCOLS: usize> Default for HsDbscan<NCOLS> {
    fn default() -> Self {
        Self {
            raster_res: 1.0,
            proximity: Default::default(),
            min_pts: 10,
        }
    }
}

impl<const NCOLS: usize> HsDbscan<NCOLS> {
    pub fn cluster(&self, input: &MatrixType<NCOLS>) {
        // Create a proximity calculator
        Proximity::new(input, &self.proximity);
    }
}
