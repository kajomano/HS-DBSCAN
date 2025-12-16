//! To run the benchmarks:
//! ```
//! cargo bench
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, rng};
use rf_dbscan::{
    RfDbscan,
    proximity::{Norm, Proximity},
    types::{IndexType, MatrixType},
};
use std::hint::black_box;

fn generate_uniform<const NCOLS: usize>(n_points: IndexType) -> MatrixType<NCOLS> {
    let mut rng = rng();
    MatrixType::<NCOLS>::from_fn(n_points as usize, |_, _| rng.random_range(-10.0..10.0))
}

fn benchmark_uniform(c: &mut Criterion) {
    let rf_dbscan = RfDbscan {
        raster_res: 1.0,
        eps: Proximity::new(10.0, Norm::L2),
        min_pts: 10,
    };

    let input = generate_uniform::<2>(1000);

    c.bench_function("uniform 1000", |b| {
        b.iter(|| rf_dbscan.cluster(black_box(&input)))
    });
}

criterion_group!(benches, benchmark_uniform);
criterion_main!(benches);
