//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use rf_dbscan::{dbscan::RfDbscan, generate::generate_uniform};
use std::{hint::black_box, time::Duration};

fn benchmark_uniform(c: &mut Criterion) {
    let dbscan = RfDbscan::default();

    let mut group = c.benchmark_group("uniform");
    group.measurement_time(Duration::from_secs(10));

    let input = generate_uniform::<2>(100, 10.0);
    group.bench_function("100", |b| b.iter(|| black_box(dbscan.cluster(&input))));

    let input = generate_uniform::<2>(1000, 10.0);
    group.bench_function("1000", |b| b.iter(|| black_box(dbscan.cluster(&input))));

    group.finish();
}

criterion_group!(benches, benchmark_uniform);
criterion_main!(benches);
