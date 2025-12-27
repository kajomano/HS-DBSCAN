//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use hs_dbscan::{
    HsDbscanConfig,
    generate::{Generate, Uniform},
    proximity::{MatrixProximity, Proximity, ProximityConfig},
};
use std::{hint::black_box, time::Duration};

fn benchmark_proximity_init(
    proximity_config: &ProximityConfig,
    generator: &impl Generate,
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(format!("prox_init_{}_{}", proximity_config.norm, generator));
    group.measurement_time(Duration::from_secs(20));

    // TODO: move these to macros/functions
    let input = generator.generate::<2>(100);
    group.bench_function("100", |b| {
        b.iter(|| black_box(MatrixProximity::new(&input, &proximity_config)))
    });

    let input = generator.generate::<2>(1000);
    group.bench_function("1000", |b| {
        b.iter(|| black_box(MatrixProximity::new(&input, &proximity_config)))
    });

    let input = generator.generate::<2>(10000);
    group.bench_function("10000", |b| {
        b.iter(|| black_box(MatrixProximity::new(&input, &proximity_config)))
    });

    group.finish();
}

fn benchmark_proximity_query(
    proximity_config: &ProximityConfig,
    generator: &impl Generate,
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group(format!(
        "prox_query_{}_{}",
        proximity_config.norm, generator
    ));
    group.measurement_time(Duration::from_secs(5));

    let inner_fn = |proximity: &MatrixProximity, n: usize| {
        for idx in 0..n {
            proximity.query(idx);
        }
    };

    // TODO: move these to macros/functions
    let proximity = MatrixProximity::new(&generator.generate::<2>(100), &proximity_config);
    group.bench_function("100", |b| b.iter(|| black_box(inner_fn(&proximity, 100))));

    let proximity = MatrixProximity::new(&generator.generate::<2>(1000), &proximity_config);
    group.bench_function("1000", |b| b.iter(|| black_box(inner_fn(&proximity, 1000))));

    let proximity = MatrixProximity::new(&generator.generate::<2>(10000), &proximity_config);
    group.bench_function("10000", |b| {
        b.iter(|| black_box(inner_fn(&proximity, 10000)))
    });

    group.finish();
}

fn benchmark_proximity(c: &mut Criterion) {
    let config = HsDbscanConfig::default();
    let generator = Uniform { extent: 10.0 };
    benchmark_proximity_init(&config.proximity, &generator, c);
    benchmark_proximity_query(&config.proximity, &generator, c);
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
