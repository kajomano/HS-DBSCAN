use nalgebra::{Const, Dyn, OMatrix};
use rf_dbscan::types::{FloatType, IndexType, MatrixType};

// TODO: make this generic
enum Distance {
    L2(FloatType),
}

struct RfDbscan<const NCOLS: usize> {
    raster_res: FloatType,
    eps: Distance,
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
        eps: Distance::L2(1.0),
        min_pts: 10,
    };

    let input = OMatrix::<FloatType, Dyn, Const<2>>::repeat(10, 0.0);

    rf_dbscan.cluster(input);
}
