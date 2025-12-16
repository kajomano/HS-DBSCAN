use nalgebra::{Const, Dyn, OMatrix};
use rf_dbscan::{
    proximity::{Norm, Proximity},
    types::{FloatType, IndexType, MatrixType},
};

// https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html

struct RfDbscan<const NCOLS: usize> {
    raster_res: FloatType,
    eps: Proximity<NCOLS>,
    min_pts: usize,
}

impl<const NCOLS: usize> RfDbscan<NCOLS> {
    fn cluster(&self, input: MatrixType<NCOLS>) -> Vec<IndexType> {
        // https://github.com/savish/dbscan

        vec![]
    }
}

fn main() {
    let rf_dbscan = RfDbscan {
        raster_res: 1.0,
        eps: Proximity::new(10.0, Norm::L2),
        min_pts: 10,
    };

    let input = OMatrix::<FloatType, Dyn, Const<2>>::repeat(10, 0.0);

    rf_dbscan.cluster(input);
}
