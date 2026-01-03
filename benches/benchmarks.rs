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
) {
    let mut group = c.benchmark_group(format!("rast_init_{}_{}", raster_res, generator));
    group.measurement_time(Duration::from_secs(5));

    macro_rules! rasterizer_init {
        ($n:literal) => {
            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(Rasterizer::new(&generator.generate($n), raster_res)))
            });
        };
    }

    rasterizer_init!(100);
    rasterizer_init!(1000);
    rasterizer_init!(10000);

    group.finish();
}

#[allow(dead_code)]
fn benchmark_rasterizer_map(
    raster_res: FloatType,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(format!("rast_map_{}_{}", raster_res, generator));
    group.measurement_time(Duration::from_secs(5));

    macro_rules! rasterizer_map {
        ($n:literal) => {
            let rasterizer = Rasterizer::new(&generator.generate($n), raster_res);
            let (centroids, _) = rasterizer.rasterize();
            let cluster_ids = IndexVectorType::zeros(centroids.nrows());

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(rasterizer.map_back(&cluster_ids)))
            });
        };
    }

    rasterizer_map!(100);
    rasterizer_map!(1000);
    rasterizer_map!(10000);

    group.finish();
}

fn benchmark_proximity_init(
    proximity_config: &ProximityConfig,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(format!("prox_init_{}_{}", proximity_config.norm, generator));
    group.measurement_time(Duration::from_secs(10));

    macro_rules! proximity_init {
        ($n:literal) => {
            let input = generator.generate($n);
            let weights = IndexVectorType::repeat($n, 1);

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(MatrixProximity::new(&input, &weights, &proximity_config)))
            });
        };
    }

    proximity_init!(100);
    proximity_init!(1000);
    proximity_init!(10000);

    group.finish();
}

#[allow(dead_code)]
fn benchmark_proximity_query(
    proximity_config: &ProximityConfig,
    generator: &impl Generate<2>,
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(format!(
        "prox_query_{}_{}",
        proximity_config.norm, generator
    ));
    group.measurement_time(Duration::from_secs(5));

    let inner_fn = |prox: &MatrixProximity, n: usize| {
        for idx in 0..n {
            prox.query(idx);
        }
    };

    macro_rules! proximity_query {
        ($n:literal) => {
            let prox = MatrixProximity::new(
                &generator.generate($n),
                &IndexVectorType::repeat($n, 1),
                &proximity_config,
            );

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(inner_fn(&prox, $n)))
            });
        };
    }

    proximity_query!(100);
    proximity_query!(1000);
    proximity_query!(10000);

    group.finish();
}

fn benchmark_dbscan(config: &HsDbscanConfig, generator: &impl Generate<2>, c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("dbscan_{}", generator));
    group.measurement_time(Duration::from_secs(10));

    macro_rules! dbscan {
        ($n:literal) => {
            let prox = MatrixProximity::new(
                &generator.generate($n),
                &IndexVectorType::repeat($n, 1),
                &config.proximity,
            );

            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(dbscan(&prox, config.min_pts)))
            });
        };
    }

    dbscan!(100);
    dbscan!(1000);
    dbscan!(10000);

    group.finish();
}

fn benchmark_e2e(config: &HsDbscanConfig, generator: &impl Generate<2>, c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("e2e_{}", generator));
    group.measurement_time(Duration::from_secs(10));

    macro_rules! e2e {
        ($n:literal) => {
            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(hs_dbscan(&generator.generate($n), config)))
            });
        };
    }

    e2e!(100);
    e2e!(1000);
    e2e!(10000);

    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    let config = HsDbscanConfig::test_default();

    // Rasterizer
    benchmark_rasterizer_init(config.raster_res.unwrap(), &UniformBox::test_default(), c);
    // benchmark_rasterizer_map(config.raster_res.unwrap(), &UniformBox::test_default(), c);

    // Proximity
    benchmark_proximity_init(&config.proximity, &UniformBox::test_default(), c);
    // benchmark_proximity_query(&config.proximity, &UniformBox::test_default(), c);

    // DBSCAN
    benchmark_dbscan(&config, &UniformBox::test_default(), c);

    // End-to-end
    benchmark_e2e(&config, &UniformBox::test_default(), c);
    benchmark_e2e(&config, &UniformSphere::test_default(), c);
    benchmark_e2e(&config, &TwoUniformSpheres::test_default(), c);
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);

// =====================================================================================================================

// f64

// rast_init_1_uniformbox/100           time:   [2.6827 µs 2.6870 µs 2.6918 µs]
// rast_init_1_uniformbox/1000          time:   [16.653 µs 16.680 µs 16.712 µs]
// rast_init_1_uniformbox/10000         time:   [212.78 µs 214.05 µs 215.48 µs]

// prox_init_L2Squared_uniformbox/100   time:   [10.446 µs 10.458 µs 10.471 µs]
// prox_init_L2Squared_uniformbox/1000  time:   [1.1359 ms 1.1371 ms 1.1384 ms]
// prox_init_L2Squared_uniformbox/10000 time:   [140.88 ms 141.08 ms 141.29 ms]

// dbscan_uniformbox/100                time:   [12.454 µs 12.491 µs 12.529 µs]
// dbscan_uniformbox/1000               time:   [2.3946 ms 2.4066 ms 2.4186 ms]
// dbscan_uniformbox/10000              time:   [277.25 ms 277.63 ms 278.00 ms]

// e2e_uniformbox/100                   time:   [5.7981 µs 5.8055 µs 5.8135 µs]
// e2e_uniformbox/1000                  time:   [21.093 µs 21.115 µs 21.138 µs]
// e2e_uniformbox/10000                 time:   [229.47 µs 230.70 µs 231.91 µs]

// e2e_uniformsphere/100                time:   [3.2413 µs 3.2453 µs 3.2500 µs]
// e2e_uniformsphere/1000               time:   [26.385 µs 26.408 µs 26.433 µs]
// e2e_uniformsphere/10000              time:   [280.30 µs 280.88 µs 281.54 µs]

// e2e_2uniformspheres/100              time:   [5.3867 µs 5.3925 µs 5.3986 µs]
// e2e_2uniformspheres/1000             time:   [35.255 µs 35.297 µs 35.350 µs]
// e2e_2uniformspheres/10000            time:   [328.87 µs 329.72 µs 330.54 µs]
