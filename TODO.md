## High-level
- [ ] Write dumb implementation
- [ ] Write unit tests
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
- [ ] Try to avoid the temp vector
  - [ ] Redo benchmark on target hardware
- [ ] Handle weighted points

- [x] Add query function to proximity
- [x] Add unit tests for the proximity calcs
- [ ] Add unit tests for the dbscan

- [x] Pass the data generation as a function arg
- [x] Add query benchmarks

- [ ] Move things into `pub(crate)` wherever possible


## Issues
- Norm variants don't change the performance
  - not sure what it was, now they change significantly


## Housekeeping
- [ ] Add LICENSE
- [ ] Add README
- [ ] Clean up TODOs
