use crate::types::{FloatType, IndexType};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Top-level configuration struct for HS-DBSCAN.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HsDbscanConfig {
    pub raster_res: Option<FloatType>,
    pub proximity: ProximityConfig,
    pub min_pts: IndexType,
}

/// Configuration for proximity calculation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ProximityConfig {
    pub eps: FloatType,
    pub norm: NormConfig,
}

/// Different supported [norms](https://docs.rs/nalgebra/latest/nalgebra/base/trait.Norm.html).
#[derive(Clone, Copy, Debug, Display, Serialize, Deserialize)]
pub enum NormConfig {
    L1,
    L2,
    L2Squared,
    Linf,
}

pub mod test {
    use crate::config::{HsDbscanConfig, NormConfig, ProximityConfig};

    /// Trait to define default configuration used in unit tests and benchmarks
    pub trait TestDefault {
        fn test_default() -> Self;
    }

    impl TestDefault for HsDbscanConfig {
        fn test_default() -> Self {
            Self {
                raster_res: Some(0.3),
                proximity: TestDefault::test_default(),
                min_pts: 10,
            }
        }
    }

    impl TestDefault for ProximityConfig {
        fn test_default() -> Self {
            Self {
                eps: 1.0,
                norm: TestDefault::test_default(),
            }
        }
    }

    impl TestDefault for NormConfig {
        fn test_default() -> Self {
            Self::L2Squared
        }
    }

    #[cfg(test)]
    mod test {
        use crate::config::{HsDbscanConfig, test::TestDefault};
        use serde::Serialize;
        use serde_json::{Serializer, ser::PrettyFormatter};
        use std::{fs::File, io::BufWriter, path::Path};

        #[test]
        fn write_default_config_json() {
            let mut writer = BufWriter::new(File::create(&Path::new("./config.json")).unwrap());
            let mut serializer =
                Serializer::with_formatter(&mut writer, PrettyFormatter::with_indent(b"\t"));

            Serialize::serialize(&HsDbscanConfig::test_default(), &mut serializer).unwrap();
        }
    }
}
