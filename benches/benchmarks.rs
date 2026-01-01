//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use hs_dbscan::{
    HsDbscanConfig,
    dbscan::dbscan,
    generate::{Generate, UniformBox},
    proximity::{MatrixProximity, Proximity, ProximityConfig},
};
use std::{hint::black_box, time::Duration};

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
            group.bench_function(format!("{}", $n), |b| {
                b.iter(|| black_box(MatrixProximity::new(&input, &proximity_config)))
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
            let prox = MatrixProximity::new(&generator.generate($n), &proximity_config);
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
            let prox = MatrixProximity::new(&generator.generate($n), &config.proximity);

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
    let config = HsDbscanConfig::default();

    // // Proximity
    // benchmark_proximity_init(
    //     &config.proximity,
    //     &UniformBox {
    //         center: [5.0, 5.0],
    //         size: 5.0,
    //     },
    //     c,
    // );
    // benchmark_proximity_query(
    //     &config.proximity,
    //     &UniformBox {
    //         center: [5.0, 5.0],
    //         size: 5.0,
    //     },
    //     c,
    // );

    // DBSCAN
    benchmark_dbscan(
        &config,
        &UniformBox {
            center: [5.0, 5.0],
            size: 5.0,
        },
        c,
    );
}

criterion_group!(benches, benchmark_proximity);
criterion_main!(benches);

// =====================================================================================================================

// f32
// prox_init_L2_uniform/100   time:   [9.2024 µs 9.2225 µs 9.2439 µs]
// prox_init_L2_uniform/1000  time:   [930.24 µs 931.23 µs 932.27 µs]
// prox_init_L1_uniform/100   time:   [8.1238 µs 8.1376 µs 8.1517 µs]
// prox_init_L1_uniform/1000  time:   [880.65 µs 882.80 µs 885.33 µs]

// f64
// prox_init_L2_uniform/100   time:   [12.128 µs 12.146 µs 12.165 µs]
// prox_init_L2_uniform/1000  time:   [1.2793 ms 1.2804 ms 1.2816 ms]
// prox_init_L1_uniform/100   time:   [8.7750 µs 8.7929 µs 8.8117 µs]
// prox_init_L1_uniform/1000  time:   [941.61 µs 942.97 µs 944.43 µs]

// dbscan_L2_uniformbox/100   time:   [12.606 µs 12.633 µs 12.661 µs]
// dbscan_L2_uniformbox/1000  time:   [2.4429 ms 2.4510 ms 2.4584 ms]
// dbscan_L2_uniformbox/10000 time:   [281.28 ms 281.54 ms 281.79 ms]
