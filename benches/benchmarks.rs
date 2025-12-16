//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use rf_dbscan::{
    RfDbscan,
    generate::generate_uniform,
    proximity::{Norm, Proximity},
};
use std::hint::black_box;

fn benchmark_uniform(c: &mut Criterion) {
    let rf_dbscan = RfDbscan {
        raster_res: 1.0,
        eps: Proximity::new(10.0, Norm::L2),
        min_pts: 10,
    };

    let input = generate_uniform::<2>(1000);

    c.bench_function("uniform 1000", |b| {
        b.iter(|| black_box(rf_dbscan.cluster(&input)))
    });
}

criterion_group!(benches, benchmark_uniform);
criterion_main!(benches);
