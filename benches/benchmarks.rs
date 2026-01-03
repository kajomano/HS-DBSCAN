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
    benchmark_rasterizer_map(config.raster_res.unwrap(), &UniformBox::test_default(), c);

    // Proximity
    benchmark_proximity_init(&config.proximity, &UniformBox::test_default(), c);
    benchmark_proximity_query(&config.proximity, &UniformBox::test_default(), c);

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

// rast_init_1_uniformbox/100    time:   [2.8790 µs 2.9260 µs 2.9690 µs]
// rast_init_1_uniformbox/1000   time:   [16.550 µs 16.575 µs 16.603 µs]
// rast_init_1_uniformbox/10000  time:   [215.03 µs 216.36 µs 217.75 µs]

// prox_init_L2_uniformbox/100   time:   [13.791 µs 13.802 µs 13.813 µs]
// prox_init_L2_uniformbox/1000  time:   [1.4627 ms 1.4645 ms 1.4664 ms]
// prox_init_L2_uniformbox/10000 time:   [178.93 ms 179.14 ms 179.35 ms]

// dbscan_uniformbox/100         time:   [12.873 µs 12.906 µs 12.939 µs]
// dbscan_uniformbox/1000        time:   [2.4927 ms 2.4990 ms 2.5051 ms]
// dbscan_uniformbox/10000       time:   [286.71 ms 287.09 ms 287.50 ms]

// e2e_uniformbox/100            time:   [10.427 µs 10.437 µs 10.448 µs]
// e2e_uniformbox/1000           time:   [38.645 µs 38.685 µs 38.730 µs]
// e2e_uniformbox/10000          time:   [307.81 µs 308.09 µs 308.38 µs]

// e2e_uniformsphere/100         time:   [4.1890 µs 4.1933 µs 4.1979 µs]
// e2e_uniformsphere/1000        time:   [35.013 µs 35.042 µs 35.073 µs]
// e2e_uniformsphere/10000       time:   [343.49 µs 343.82 µs 344.16 µs]

// e2e_2uniformspheres/100       time:   [11.076 µs 11.089 µs 11.104 µs]
// e2e_2uniformspheres/1000      time:   [68.490 µs 68.550 µs 68.616 µs]
// e2e_2uniformspheres/10000     time:   [507.15 µs 507.54 µs 507.97 µs]
