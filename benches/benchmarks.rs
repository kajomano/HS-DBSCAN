//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use rf_dbscan::{
    dbscan::RfDbscan,
    generate::generate_uniform,
    proximity::{NormConfig, ProximityConfig},
};
use std::{hint::black_box, time::Duration};

// TODO: Pass the generation function as arg
fn benchmark_proximity_init_uniform(dbscan: &RfDbscan<2>, c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("prox_init_{}_uniform", dbscan.proximity.norm()));
    group.measurement_time(Duration::from_secs(10));

    let input = generate_uniform(100, 10.0);
    group.bench_function("100", |b| b.iter(|| black_box(dbscan.cluster(&input))));

    let input = generate_uniform(1000, 10.0);
    group.bench_function("1000", |b| b.iter(|| black_box(dbscan.cluster(&input))));

    group.finish();
}

fn benchmark_proximity_init(c: &mut Criterion) {
    let dbscan = RfDbscan::default();
    benchmark_proximity_init_uniform(&dbscan, c);

    let dbscan = RfDbscan {
        proximity: ProximityConfig::new(ProximityConfig::default().eps(), NormConfig::L1),
        ..Default::default()
    };
    benchmark_proximity_init_uniform(&dbscan, c);
}

criterion_group!(benches, benchmark_proximity_init);
criterion_main!(benches);

// =====================================================================================================================

// f32
// prox_init_L2_uniform/100   time:   [8.4054 µs 8.4150 µs 8.4251 µs]
// prox_init_L2_uniform/1000  time:   [1.1128 ms 1.1141 ms 1.1154 ms]
// prox_init_L1_uniform/100   time:   [6.5510 µs 6.5664 µs 6.5825 µs]
// prox_init_L1_uniform/1000  time:   [856.29 µs 857.79 µs 859.51 µs]

// f64
// prox_init_L2_uniform/100   time:   [15.297 µs 15.309 µs 15.323 µs]
// prox_init_L2_uniform/1000  time:   [1.9357 ms 1.9382 ms 1.9408 ms]
// prox_init_L1_uniform/100   time:   [7.7790 µs 7.7917 µs 7.8078 µs]
// prox_init_L1_uniform/1000  time:   [1.4314 ms 1.4364 ms 1.4420 ms]
