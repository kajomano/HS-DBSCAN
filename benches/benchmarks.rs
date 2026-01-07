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
    dbscan::dbscan,
    generate::{Generate, TwoUniformSpheres, UniformBox, UniformSphere},
    hs_dbscan,
    proximity::{MatrixProximity, Proximity},
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

fn benchmark_proximity_init(
    proximity_config: &ProximityConfig,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) -> Result<()> {
    let mut group = c.benchmark_group(format!("prox_init_{}_{}", proximity_config.norm, generator));
    group.measurement_time(Duration::from_secs(10));

    macro_rules! proximity_init {
        ($n:literal) => {
            let input = generator.generate($n)?;
            let weights = IndexVectorType::repeat($n, 1);

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| {
                    black_box(MatrixProximity::new(&input, &weights, &proximity_config).unwrap())
                })
            });
        };
    }

    proximity_init!(100);
    proximity_init!(1000);
    proximity_init!(10000);

    group.finish();

    Ok(())
}

#[allow(dead_code)]
fn benchmark_proximity_query(
    proximity_config: &ProximityConfig,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) -> Result<()> {
    let mut group = c.benchmark_group(format!(
        "prox_query_{}_{}",
        proximity_config.norm, generator
    ));
    group.measurement_time(Duration::from_secs(5));

    let inner_fn = |prox: &MatrixProximity, n: usize| {
        for idx in 0..n {
            unsafe {
                prox.query(idx);
            }
        }
    };

    macro_rules! proximity_query {
        ($n:literal) => {
            let prox = MatrixProximity::new(
                &generator.generate($n)?,
                &IndexVectorType::repeat($n, 1),
                &proximity_config,
            )?;

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(inner_fn(&prox, $n)))
            });
        };
    }

    proximity_query!(100);
    proximity_query!(1000);
    proximity_query!(10000);

    group.finish();

    Ok(())
}

fn benchmark_dbscan(
    config: &HsDbscanConfig,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) -> Result<()> {
    let mut group = c.benchmark_group(format!("dbscan_{}", generator));
    group.measurement_time(Duration::from_secs(10));

    macro_rules! dbscan {
        ($n:literal) => {
            let prox = MatrixProximity::new(
                &generator.generate($n)?,
                &IndexVectorType::repeat($n, 1),
                &config.proximity,
            )?;

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(dbscan(&prox, config.min_pts)))
            });
        };
    }

    dbscan!(100);
    dbscan!(1000);
    dbscan!(10000);

    group.finish();

    Ok(())
}

fn benchmark_e2e(
    config: &HsDbscanConfig,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) -> Result<()> {
    let mut group = c.benchmark_group(format!("e2e_{}", generator));
    group.measurement_time(Duration::from_secs(10));

    macro_rules! e2e {
        ($n:literal, $min_pts:literal) => {
            let input = generator.generate($n)?;
            let config = HsDbscanConfig {
                min_pts: $min_pts,
                ..*config
            };

            group.bench_function(format!("{}_{}", $n, $min_pts), |b| {
                b.iter(|| black_box(hs_dbscan(&input, &config).unwrap()))
            });
        };
    }

    e2e!(100, 10);
    e2e!(1000, 70);
    e2e!(10000, 500);

    group.finish();

    Ok(())
}

fn benchmarks(c: &mut Criterion) {
    let config = HsDbscanConfig::test_default();

    // Rasterizer
    benchmark_rasterizer_init(config.raster_res.unwrap(), &UniformBox::test_default(), c).unwrap();
    // benchmark_rasterizer_map(config.raster_res.unwrap(), &UniformBox::test_default(), c).unwrap();

    // Proximity
    benchmark_proximity_init(&config.proximity, &UniformBox::test_default(), c).unwrap();
    // benchmark_proximity_query(&config.proximity, &UniformBox::test_default(), c).unwrap();

    // DBSCAN
    benchmark_dbscan(&config, &UniformBox::test_default(), c).unwrap();

    // End-to-end
    benchmark_e2e(&config, &UniformBox::test_default(), c).unwrap();
    benchmark_e2e(&config, &UniformSphere::test_default(), c).unwrap();
    benchmark_e2e(&config, &TwoUniformSpheres::test_default(), c).unwrap();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);

// =====================================================================================================================

// x86_64

// rast_init_0.7_uniformbox/100         time:   [2.1480 µs 2.1518 µs 2.1562 µs]
// rast_init_0.7_uniformbox/1000        time:   [9.9839 µs 9.9963 µs 10.011 µs]
// rast_init_0.7_uniformbox/10000       time:   [78.082 µs 78.230 µs 78.399 µs]

// prox_init_L2_uniformbox/100          time:   [8.6554 µs 8.6672 µs 8.6804 µs]
// prox_init_L2_uniformbox/1000         time:   [903.54 µs 904.75 µs 906.08 µs]
// prox_init_L2_uniformbox/10000        time:   [116.14 ms 116.48 ms 116.84 ms]

// dbscan_uniformbox/100                time:   [5.8543 µs 5.8648 µs 5.8769 µs]
// dbscan_uniformbox/1000               time:   [88.161 µs 88.246 µs 88.339 µs]
// dbscan_uniformbox/10000              time:   [137.94 ms 138.08 ms 138.24 ms]

// e2e_uniformbox/100_10                time:   [13.238 µs 13.256 µs 13.275 µs]
// e2e_uniformbox/1000_70               time:   [60.840 µs 60.916 µs 60.999 µs]
// e2e_uniformbox/10000_500             time:   [143.35 µs 143.57 µs 143.81 µs]

// e2e_uniformsphere/100_10             time:   [1.0498 µs 1.0517 µs 1.0541 µs]
// e2e_uniformsphere/1000_70            time:   [7.7134 µs 7.7250 µs 7.7387 µs]
// e2e_uniformsphere/10000_500          time:   [75.045 µs 75.137 µs 75.239 µs]

// e2e_2uniformspheres/100_10           time:   [9.0921 µs 9.1066 µs 9.1228 µs]
// e2e_2uniformspheres/1000_70          time:   [53.678 µs 53.770 µs 53.875 µs]
// e2e_2uniformspheres/10000_500        time:   [163.07 µs 163.31 µs 163.59 µs]

// arm64

// rast_init_0.7_uniformbox/100         time:   [2.7051 µs 2.7275 µs 2.7644 µs]
// rast_init_0.7_uniformbox/1000        time:   [13.596 µs 13.644 µs 13.692 µs]
// rast_init_0.7_uniformbox/10000       time:   [122.26 µs 123.31 µs 124.29 µs]

// prox_init_L2_uniformbox/100          time:   [11.352 µs 11.401 µs 11.472 µs]
// prox_init_L2_uniformbox/1000         time:   [825.85 µs 830.37 µs 836.11 µs]
// prox_init_L2_uniformbox/10000        time:   [181.64 ms 182.42 ms 183.44 ms]

// dbscan_uniformbox/100                time:   [9.2464 µs 9.3014 µs 9.3709 µs]
// dbscan_uniformbox/1000               time:   [144.95 µs 145.21 µs 145.50 µs]
// dbscan_uniformbox/10000              time:   [246.26 ms 246.82 ms 247.48 ms]

// e2e_uniformbox/100_10                time:   [15.600 µs 15.610 µs 15.620 µs]
// e2e_uniformbox/1000_70               time:   [80.985 µs 81.439 µs 82.001 µs]
// e2e_uniformbox/10000_500             time:   [152.57 µs 153.14 µs 153.87 µs]

// e2e_uniformsphere/100_10             time:   [957.83 ns 964.58 ns 972.44 ns]
// e2e_uniformsphere/1000_70            time:   [6.5056 µs 6.5323 µs 6.5703 µs]
// e2e_uniformsphere/10000_500          time:   [64.627 µs 64.801 µs 65.026 µs]

// e2e_2uniformspheres/100_10           time:   [12.812 µs 12.883 µs 12.972 µs]
// e2e_2uniformspheres/1000_70          time:   [72.641 µs 72.818 µs 73.028 µs]
// e2e_2uniformspheres/10000_500        time:   [191.33 µs 191.85 µs 192.51 µs]
