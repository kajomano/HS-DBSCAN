//! To run the benchmarks:
//! ```
//! cargo run --release
//! ```

// https://github.com/savish/dbscan

use hs_dbscan::{
    HsDbscanConfig,
    generate::{Generate, Uniform},
    hs_dbscan,
};

fn main() {
    let config = HsDbscanConfig::default();
    let input = Uniform { extent: 10.0 }.generate::<2>(5);

    hs_dbscan(&input, &config);
}
