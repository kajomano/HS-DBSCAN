//! To run the benchmarks:
//! ```
//! cargo run --release
//! ```

// https://github.com/savish/dbscan

use rf_dbscan::{
    dbscan::RfDbscan,
    generate::generate_uniform,
    proximity::{Norm, Proximity},
};

fn main() {
    let rf_dbscan = RfDbscan {
        raster_res: 1.0,
        eps: Proximity::new(10.0, Norm::L2),
        min_pts: 10,
    };

    let input = generate_uniform::<2>(1000);

    rf_dbscan
        .cluster(&input)
        .iter()
        .for_each(|row| println!("{:?}", row));
}
