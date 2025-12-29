//! To run the benchmarks:
//! ```
//! cargo run --release
//! ```

// https://github.com/savish/dbscan

use hs_dbscan::{
    HsDbscanConfig,
    generate::{Generate, UniformBox},
    hs_dbscan,
};

fn main() {
    let config = HsDbscanConfig::default();
    let input = UniformBox {
        center: [5.0, 5.0],
        size: 5.0,
    }
    .generate(5);

    hs_dbscan(&input, &config);
}
