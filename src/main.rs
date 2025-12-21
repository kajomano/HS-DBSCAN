//! To run the benchmarks:
//! ```
//! cargo run --release
//! ```

// https://github.com/savish/dbscan

use hs_dbscan::{
    dbscan::HsDbscan,
    generate::{Generate, Uniform},
    proximity::{NormConfig, ProximityConfig},
};

fn main() {
    let hs_dbscan = HsDbscan::<2> {
        raster_res: 1.0,
        proximity: ProximityConfig::new(10.0, NormConfig::L2),
        min_pts: 10,
    };

    let input = Uniform { extent: 10.0 }.generate(5);
    hs_dbscan.cluster(&input);
}
