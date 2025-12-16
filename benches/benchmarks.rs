//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use rf_dbscan::{
    dbscan::RfDbscan,
    generate::generate_uniform,
    proximity::{Norm, Proximity},
};
use std::{hint::black_box, time::Duration};

fn benchmark_uniform(c: &mut Criterion) {
    let mut group = c.benchmark_group("My Group");
    group.measurement_time(Duration::from_secs(10));

    let rf_dbscan = RfDbscan::<2> {
        raster_res: 1.0,
        eps: Proximity::new(10.0, Norm::L2),
        min_pts: 10,
    };

    let input = generate_uniform(1000);

    group.bench_function("uniform 1000", |b| {
        b.iter(|| black_box(rf_dbscan.cluster(&input)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_uniform);
criterion_main!(benches);
