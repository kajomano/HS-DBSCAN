//! To run the application:
//! ```
//! cargo run --release
//! ```

use eyre::Result;
use hs_dbscan::{
    config::HsDbscanConfig,
    generate::{Generate, UniformBox},
    hs_dbscan,
};
use std::{fs::File, path::Path};

fn main() -> Result<()> {
    let config: HsDbscanConfig = serde_json::from_reader(File::open(Path::new("./config.json"))?)?;

    println!("{:?}", config);

    let input = UniformBox {
        center: [5.0, 5.0],
        size: 5.0,
    }
    .generate(5)?;

    hs_dbscan(&input, &config)?;

    Ok(())
}
