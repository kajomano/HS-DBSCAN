## High-level
- [ ] Write dumb implementation
- [ ] Write unit tests
- [ ] Write benchmarks
- [ ] Write python bindings?
- Extend with:
    - [x] Linf distance
    - [ ] Rasterization
    - [ ] KD-tree


## Low-level
- [x] Make proximity calc batched
- [x] Add 1 benchmark
- [ ] Add unit tests for the proximity calcs

- [ ] Add more benchmarks
- [ ] Explore: https://www.nalgebra.rs/docs/user_guide/performance_tricks/
- [ ] Explore SIMD -> Try hand-rolling the dist calc yourself w/ std::simd (below)

## Issues
- Norm variants don't change the performance

## Housekeeping
- [ ] Add LICENSE
- [ ] Add README

---

```rust
use std::simd::{f32x4, Simd}; // Or packed_simd::* for older Rust

fn vector_length_simd(vec: &[f32]) -> f32 {
    let mut sum_sq = f32x4::splat(0.0); // Vector of four zeros
    for chunk in vec.chunks_exact(4) {
        let v = f32x4::from_slice(chunk); // Load 4 floats
        sum_sq += v * v; // Parallel square and add
    }
    let total_sum = sum_sq.reduce_sum(); // Sum the vector's elements
    // Handle remaining elements if len % 4 != 0 (loop or scalar)
    // ...
    total_sum.sqrt() // Final square root
}
```