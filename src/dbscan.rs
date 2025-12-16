use crate::{
    proximity::Proximity,
    types::{FloatType, MatrixType},
};

pub struct RfDbscan<const NCOLS: usize> {
    pub raster_res: FloatType,
    pub eps: Proximity<NCOLS>,
    pub min_pts: usize,
}

impl<const NCOLS: usize> RfDbscan<NCOLS> {
    pub fn cluster(&self, input: &MatrixType<NCOLS>) -> Vec<Vec<bool>> {
        // Calculate every distance to every other distance as a dummy
        input
            .row_iter()
            .map(|anchor| self.eps.within_proximity(&anchor, input))
            .collect()
    }
}
