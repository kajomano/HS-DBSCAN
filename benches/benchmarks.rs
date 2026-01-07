//! To run the benchmarks:
//! ```
//! cargo bench
//! ```
//! Filter for the specific category name with:
//! ```
//! cargo bench [category]
//! ```
//! Available categories:
//! - rast
//! - prox
//! - dbscan
//! - e2e

use criterion::{Criterion, criterion_group, criterion_main};
use eyre::Result;
use hs_dbscan::{
    config::{HsDbscanConfig, ProximityConfig, test::TestDefault},
    // dbscan::dbscan,
    generate::{Generate, TwoUniformSpheres, UniformBox, UniformSphere},
    // hs_dbscan,
    // proximity::{MatrixProximity, Proximity},
    rasterizer::Rasterizer,
    types::{FloatType, IndexVectorType},
};
use std::{hint::black_box, time::Duration};

fn benchmark_rasterizer_init(
    raster_res: FloatType,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) -> Result<()> {
    let mut group = c.benchmark_group(format!("rast_init_{}_{}", raster_res, generator));
    group.measurement_time(Duration::from_secs(5));

    macro_rules! rasterizer_init {
        ($n:literal) => {
            let input = generator.generate($n)?;

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(Rasterizer::new(&input, raster_res).unwrap()))
            });
        };
    }

    rasterizer_init!(100);
    rasterizer_init!(1000);
    rasterizer_init!(10000);

    group.finish();

    Ok(())
}

#[allow(dead_code)]
fn benchmark_rasterizer_map(
    raster_res: FloatType,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) -> Result<()> {
    let mut group = c.benchmark_group(format!("rast_map_{}_{}", raster_res, generator));
    group.measurement_time(Duration::from_secs(5));

    macro_rules! rasterizer_map {
        ($n:literal) => {
            let rasterizer = Rasterizer::new(&generator.generate($n)?, raster_res)?;
            let (centroids, _) = rasterizer.rasterize();
            let cluster_ids = IndexVectorType::zeros(centroids.nrows());

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(rasterizer.map_back(&cluster_ids).unwrap()))
            });
        };
    }

    rasterizer_map!(100);
    rasterizer_map!(1000);
    rasterizer_map!(10000);

    group.finish();

    Ok(())
}

// fn benchmark_proximity_init(
//     proximity_config: &ProximityConfig,
//     generator: &impl Generate<2>,
//     c: &mut Criterion,
// ) -> Result<()> {
//     let mut group = c.benchmark_group(format!("prox_init_{}_{}", proximity_config.norm, generator));
//     group.measurement_time(Duration::from_secs(10));

//     macro_rules! proximity_init {
//         ($n:literal) => {
//             let input = generator.generate($n)?;
//             let weights = IndexVectorType::repeat($n, 1);

//             group.bench_function(format!("{}", $n), |b| {
//                 b.iter(|| {
//                     black_box(MatrixProximity::new(&input, &weights, &proximity_config).unwrap())
//                 })
//             });
//         };
//     }

//     proximity_init!(100);
//     proximity_init!(1000);
//     proximity_init!(10000);

//     group.finish();

//     Ok(())
// }

// #[allow(dead_code)]
// fn benchmark_proximity_query(
//     proximity_config: &ProximityConfig,
//     generator: &impl Generate<2>,
//     c: &mut Criterion,
// ) -> Result<()> {
//     let mut group = c.benchmark_group(format!(
//         "prox_query_{}_{}",
//         proximity_config.norm, generator
//     ));
//     group.measurement_time(Duration::from_secs(5));

//     let inner_fn = |prox: &MatrixProximity, n: usize| {
//         for idx in 0..n {
//             unsafe {
//                 prox.query(idx);
//             }
//         }
//     };

//     macro_rules! proximity_query {
//         ($n:literal) => {
//             let prox = MatrixProximity::new(
//                 &generator.generate($n)?,
//                 &IndexVectorType::repeat($n, 1),
//                 &proximity_config,
//             )?;

//             group.bench_function(format!("{}", $n), |b| {
//                 b.iter(|| black_box(inner_fn(&prox, $n)))
//             });
//         };
//     }

//     proximity_query!(100);
//     proximity_query!(1000);
//     proximity_query!(10000);

//     group.finish();

//     Ok(())
// }

// fn benchmark_dbscan(
//     config: &HsDbscanConfig,
//     generator: &impl Generate<2>,
//     c: &mut Criterion,
// ) -> Result<()> {
//     let mut group = c.benchmark_group(format!("dbscan_{}", generator));
//     group.measurement_time(Duration::from_secs(10));

//     macro_rules! dbscan {
//         ($n:literal) => {
//             let prox = MatrixProximity::new(
//                 &generator.generate($n)?,
//                 &IndexVectorType::repeat($n, 1),
//                 &config.proximity,
//             )?;

//             group.bench_function(format!("{}", $n), |b| {
//                 b.iter(|| black_box(dbscan(&prox, config.min_pts)))
//             });
//         };
//     }

//     dbscan!(100);
//     dbscan!(1000);
//     dbscan!(10000);

//     group.finish();

//     Ok(())
// }

// fn benchmark_e2e(
//     config: &HsDbscanConfig,
//     generator: &impl Generate<2>,
//     c: &mut Criterion,
// ) -> Result<()> {
//     let mut group = c.benchmark_group(format!("e2e_{}", generator));
//     group.measurement_time(Duration::from_secs(10));

//     macro_rules! e2e {
//         ($n:literal, $min_pts:literal) => {
//             let input = generator.generate($n)?;
//             let config = HsDbscanConfig {
//                 min_pts: $min_pts,
//                 ..*config
//             };

//             group.bench_function(format!("{}_{}", $n, $min_pts), |b| {
//                 b.iter(|| black_box(hs_dbscan(&input, &config).unwrap()))
//             });
//         };
//     }

//     e2e!(100, 10);
//     e2e!(1000, 70);
//     e2e!(10000, 500);

//     group.finish();

//     Ok(())
// }

fn benchmarks(c: &mut Criterion) {
    let config = HsDbscanConfig::test_default();

    // Rasterizer
    benchmark_rasterizer_init(config.raster_res.unwrap(), &UniformBox::test_default(), c).unwrap();
    // benchmark_rasterizer_map(config.raster_res.unwrap(), &UniformBox::test_default(), c).unwrap();

    // // Proximity
    // benchmark_proximity_init(&config.proximity, &UniformBox::test_default(), c).unwrap();
    // // benchmark_proximity_query(&config.proximity, &UniformBox::test_default(), c).unwrap();

    // // DBSCAN
    // benchmark_dbscan(&config, &UniformBox::test_default(), c).unwrap();

    // // End-to-end
    // benchmark_e2e(&config, &UniformBox::test_default(), c).unwrap();
    // benchmark_e2e(&config, &UniformSphere::test_default(), c).unwrap();
    // benchmark_e2e(&config, &TwoUniformSpheres::test_default(), c).unwrap();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);

// =====================================================================================================================

// x86_64

// rast_init_0.7_uniformbox/100         time:   [2.6149 µs 2.6197 µs 2.6248 µs]
// rast_init_0.7_uniformbox/1000        time:   [13.064 µs 13.083 µs 13.106 µs]
// rast_init_0.7_uniformbox/10000       time:   [104.80 µs 104.98 µs 105.19 µs]

// prox_init_L2Squared_uniformbox/100   time:   [10.466 µs 10.479 µs 10.493 µs]
// prox_init_L2Squared_uniformbox/1000  time:   [1.0640 ms 1.0661 ms 1.0684 ms]
// prox_init_L2Squared_uniformbox/10000 time:   [132.47 ms 132.91 ms 133.43 ms]

// dbscan_uniformbox/100                time:   [5.3446 µs 5.3514 µs 5.3585 µs]
// dbscan_uniformbox/1000               time:   [98.031 µs 98.113 µs 98.203 µs]
// dbscan_uniformbox/10000              time:   [121.30 ms 121.46 ms 121.64 ms]

// e2e_uniformbox/100_10                time:   [13.604 µs 13.626 µs 13.648 µs]
// e2e_uniformbox/1000_70               time:   [74.464 µs 74.572 µs 74.684 µs]
// e2e_uniformbox/10000_500             time:   [177.64 µs 177.92 µs 178.20 µs]

// e2e_uniformsphere/100_10             time:   [1.3227 µs 1.3241 µs 1.3257 µs]
// e2e_uniformsphere/1000_70            time:   [9.7133 µs 9.7288 µs 9.7480 µs]
// e2e_uniformsphere/10000_500          time:   [94.666 µs 94.785 µs 94.918 µs]

// e2e_2uniformspheres/100_10           time:   [11.973 µs 11.985 µs 11.999 µs]
// e2e_2uniformspheres/1000_70          time:   [54.093 µs 54.214 µs 54.324 µs]
// e2e_2uniformspheres/10000_500        time:   [187.89 µs 188.09 µs 188.31 µs]

// arm64

// rast_init_0.7_uniformbox/100         time:   [2.7051 µs 2.7275 µs 2.7644 µs]
// rast_init_0.7_uniformbox/1000        time:   [13.596 µs 13.644 µs 13.692 µs]
// rast_init_0.7_uniformbox/10000       time:   [122.26 µs 123.31 µs 124.29 µs]

// prox_init_L2Squared_uniformbox/100   time:   [12.333 µs 12.360 µs 12.392 µs]
// prox_init_L2Squared_uniformbox/1000  time:   [925.37 µs 928.65 µs 932.20 µs]
// prox_init_L2Squared_uniformbox/10000 time:   [190.36 ms 190.73 ms 191.12 ms]

// dbscan_uniformbox/100                time:   [8.6272 µs 8.6447 µs 8.6691 µs]
// dbscan_uniformbox/1000               time:   [160.34 µs 160.58 µs 160.88 µs]
// dbscan_uniformbox/10000              time:   [246.46 ms 247.06 ms 247.70 ms]

// e2e_uniformbox/100_10                time:   [17.822 µs 17.889 µs 17.967 µs]
// e2e_uniformbox/1000_70               time:   [94.207 µs 94.442 µs 94.718 µs]
// e2e_uniformbox/10000_500             time:   [221.39 µs 221.94 µs 222.53 µs]

// e2e_uniformsphere/100_10             time:   [1.3011 µs 1.3054 µs 1.3107 µs]
// e2e_uniformsphere/1000_70            time:   [9.5780 µs 9.6122 µs 9.6523 µs]
// e2e_uniformsphere/10000_500          time:   [97.499 µs 98.348 µs 99.225 µs]

// e2e_2uniformspheres/100_10           time:   [16.211 µs 16.261 µs 16.310 µs]
// e2e_2uniformspheres/1000_70          time:   [71.632 µs 71.854 µs 72.071 µs]
// e2e_2uniformspheres/10000_500        time:   [239.40 µs 240.20 µs 241.09 µs]
