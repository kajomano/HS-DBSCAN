use crate::{
    proximity::Proximity,
    types::{FloatType, MatrixType},
};

pub mod proximity;
pub mod types;

pub struct RfDbscan<const NCOLS: usize> {
    pub raster_res: FloatType,
    pub eps: Proximity<NCOLS>,
    pub min_pts: usize,
}

impl<const NCOLS: usize> RfDbscan<NCOLS> {
    pub fn cluster(&self, input: &MatrixType<NCOLS>) {
        // Calculate every distance to every other distance as a dummy
        input.row_iter().for_each(|anchor| {
            self.eps.within_proximity((&anchor).into(), input);
        });
    }
}
