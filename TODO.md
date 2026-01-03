## High-level
- [x] Write dumb implementation
- [x] Write unit tests
- [ ] Write benchmarks
- [ ] Write python bindings
- Extend with:
  - [x] Other norms
  - [ ] Rasterization
  - [ ] KD-tree


## Low-level
- [x] Make proximity calc batched
- [x] Add 1 benchmark
- [x] Try to improve perf by creating persistent output buffers
- [x] Explore SIMD: Try hand-rolling the dist calc yourself w/ `std::simd` or `wide`
  - does not give considerable speedup, better control over what's happening but painful to write everything by hand
- [x] Only calculate upper/lower triangle for the distances
- [x] Lower FP precision

- [x] Immediately calculate if distance is below eps and store that in proximity
  - [x] But don't mess up the reusability of `Norm`!
- [x] Try to avoid the temp vector
  - [x] Redo benchmark on target hardware
  - avoiding the temp speeds up very small batches (~100) by 30%, but gives a 15% slowdown on larger batches

- [x] Add query function to proximity
- [x] Add unit tests for the proximity calcs
- [x] Add unit tests for the dbscan

- [x] Pass the data generation as a function arg
- [x] Add query benchmarks

- [x] Fix mut borrow and update dbscan unit test

- [ ] Add DBSCAN benchmarks
  - [ ] Add more data distributions
  - [x] Try to tune
    - Bool seed lookup is better than `Hashset`
    - AoS is better than SoA for dbscan internal state
    - [x] w/o unsafe
      - Unsafe is a 10~20% speedup
    - Storing the idx in the state struct is a 6~13% speedup
- [x] Move repeated benchmark internals to macros/functions

- [x] Move `Default` from the config structs into another trait

- [x] Handle weighted points
  - [x] Add unit test
- [ ] Rasterization
  - [x] Mapping
  - [ ] Unit tests
    - [ ] Mapper unit test
  - [ ] Benchmarks
  - [ ] Try to tune it
    - [x] Try to change [hashing func](https://www.reddit.com/r/rust/comments/1eqhe9a/blog_i_compared_14_hashing_algorithms_on_rust/):
      - [x] [https://github.com/ogxd/gxhash]
        - not portable
      - [x] [https://github.com/hoxxep/rapidhash]
        - Small (~10%) improvement
      - [x] [https://github.com/paritytech/nohash-hasher]
        - too limiting
  - [ ] I know NCOLS at compile time, should not need to collect into `Vec` -> Try a macro
  - [ ] Centroids could also be stored

- [ ] Try converting to column-major-friendly layout for speed?

- [ ] Plots

- [ ] Clippy

## Issues
- Norm variants don't change the performance
  - not sure what it was, now they change significantly
- Smaller `RasterTypes` give a slowdown (i32, i16)

## Housekeeping
- [ ] Add LICENSE
- [ ] Add README
- [ ] Clean up TODOs
