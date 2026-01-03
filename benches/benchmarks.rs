//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use hs_dbscan::{
    config::{HsDbscanConfig, ProximityConfig, test::TestDefault},
    dbscan::dbscan,
    generate::{Generate, UniformBox},
    proximity::{MatrixProximity, Proximity},
    rasterizer::Rasterizer,
    types::{FloatType, IndexType, IndexVectorType},
};
use nalgebra::{Dyn, OVector};
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
    let mut group = c.benchmark_group(format!("dbscan_{}_{}", config.proximity.norm, generator));
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

fn benchmark_proximity(c: &mut Criterion) {
    let config = HsDbscanConfig::test_default();
    let generator = UniformBox {
        center: [5.0, 5.0],
        size: 5.0,
    };

    // Rasterizer
    benchmark_rasterizer_init(config.raster_res.unwrap(), &generator, c);
    benchmark_rasterizer_map(config.raster_res.unwrap(), &generator, c);

    // Proximity
    benchmark_proximity_init(&config.proximity, &generator, c);
    benchmark_proximity_query(&config.proximity, &generator, c);

    // DBSCAN
    benchmark_dbscan(&config, &generator, c);
}

criterion_group!(benches, benchmark_proximity);
criterion_main!(benches);

// =====================================================================================================================

// f64
// rast_init_1_uniformbox/100    time:   [5.8178 µs 5.8258 µs 5.8340 µs]
// rast_init_1_uniformbox/1000   time:   [40.630 µs 40.714 µs 40.801 µs]
// rast_init_1_uniformbox/10000  time:   [391.59 µs 392.91 µs 394.23 µs]

// prox_init_L2_uniformbox/100   time:   [13.791 µs 13.802 µs 13.813 µs]
// prox_init_L2_uniformbox/1000  time:   [1.4627 ms 1.4645 ms 1.4664 ms]
// prox_init_L2_uniformbox/10000 time:   [178.93 ms 179.14 ms 179.35 ms]

// dbscan_L2_uniformbox/100      time:   [12.873 µs 12.906 µs 12.939 µs]
// dbscan_L2_uniformbox/1000     time:   [2.4927 ms 2.4990 ms 2.5051 ms]
// dbscan_L2_uniformbox/10000    time:   [286.71 ms 287.09 ms 287.50 ms]
