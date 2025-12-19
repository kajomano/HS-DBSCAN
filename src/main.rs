//! To run the benchmarks:
//! ```
//! cargo run --release
//! ```

// https://github.com/savish/dbscan

use rf_dbscan::{
    dbscan::RfDbscan,
    generate::generate_uniform,
    proximity::{NormConfig, ProximityConfig},
};

fn main() {
    let rf_dbscan = RfDbscan::<2> {
        raster_res: 1.0,
        proximity: ProximityConfig::new(10.0, NormConfig::L2),
        min_pts: 10,
    };

    let input = generate_uniform::<2>(1000, 10.0);

    rf_dbscan.cluster(&input);
}
