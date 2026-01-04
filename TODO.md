## High-level
- [x] Write dumb implementation
- [x] Write unit tests
- [x] Write benchmarks
- [x] Write plotting
- Extend with:
  - [x] Other norms
  - [x] Rasterization
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

- [x] Add DBSCAN benchmarks
  - [x] ~~Add more data distributions~~ -> e2e benchmarks will handle it
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
- [x] Rasterization
  - [x] Mapper
  - [x] Unit tests
    - [x] Mapper unit test
  - [x] Benchmarks
    - [x] Mapper benchmark
  - [x] Try to tune it
    - [x] Try to change [hashing func](https://www.reddit.com/r/rust/comments/1eqhe9a/blog_i_compared_14_hashing_algorithms_on_rust/):
      - [x] [https://github.com/ogxd/gxhash]
        - not portable
      - -> [x] [https://github.com/hoxxep/rapidhash]
        - Small (~10%) improvement
      - [x] [https://github.com/paritytech/nohash-hasher]
        - too limiting
      - [x] [https://github.com/tkaitchuck/aHash]
        - no improvement over rapidhash
    - [x] Try a radix sort instead of hashing
      - Much slower, stopped trying
  - [x] I know NCOLS at compile time, should not need to collect into `Vec`
    - epic ~50% speedup once fixed!
  - [x] Centroids could also be stored
    - causes slowdown

- [x] e2e benchmarks
  - [x] More data distributions!

- [x] Plots
  - [x] Moar plots

- [ ] Try converting to column-major-friendly layout for speed
  - [ ] Rename NCOLS to NDIMS
- [ ] Generic proximity, use integers directly from the rasterizer

- [x] Clippy
- [x] Error handling
- [x] Docs
  - [x] README
  - [x] Docstrings
  - [x] Update benchmark results

## Issues
- [x] Norm variants don't change the performance
  - not sure what it was, now they change significantly
- [x] Smaller `RasterTypes` give a slowdown (i32, i16)
  - Fixed once I cleaned up inefficiencies in the rasterizer
- [ ] Rasterizer benchmarks are not reproducible, vary like crazy
- [x] Something doesn't work when rasterizing, visible on plots
  - Works as intended, wasn't changing the config in the correct place

## Housekeeping
- [x] Add LICENSE
- [x] Add README
- [x] Clean up TODOs
