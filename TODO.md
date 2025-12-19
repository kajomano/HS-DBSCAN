## High-level
- [ ] Write dumb implementation
- [ ] Write unit tests
- [ ] Write benchmarks
- [ ] Write python bindings?
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

- [ ] Add unit tests for the proximity calcs
- [ ] Add more benchmarks


## Issues
- Norm variants don't change the performance
  - not sure what it was, now they change significantly


## Housekeeping
- [ ] Add LICENSE
- [ ] Add README
