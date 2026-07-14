# Specified-boundary relaxation performance baseline

This compares the exact WRF v4.7.1 `relax_bdytend` wrapper and core with the
safe Rust specified-boundary relaxation capability. The matched workload
applies 238,080 five-point updates on a 256 × 256 × 40 mass grid. One outer
point is fixed, distances one through six are relaxed, and each boundary array
stores eight points normal to its side.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust 1.96.0 uses
optimization level 3, ThinLTO, and one codegen unit. Neither implementation
enables fast-math or a native CPU target. Measurements ran on an Apple M3 Max
under macOS 26.2 arm64.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran | 0.407650 ms median | 1.00× |
| Rust, 1 worker | 1.355342 ms | 3.33× slower |
| Rust, 4 workers | 0.468761 ms | 15.0% slower |
| Rust, 16 workers | 0.465916 ms | 14.3% slower |

Fortran's eleven samples ranged from 0.399020 to 0.471110 ms. Rust's bencher
dispersion was ±0.139952 ms, ±0.007256 ms, and ±0.007113 ms for one, four, and
sixteen workers, respectively.

The first exact Rust implementation selected boundary slices and side behavior
inside each stencil point and measured 3.193886 ms with one worker. Moving side
selection, boundary-slice lookup, line length, and vertical offset outside the
point loop reduced serial time by 2.36× while preserving all 5,500 oracle
values. The resulting code still shares one readable source-ordered stencil
instead of duplicating four large loop families.

The warmed default-worker path records one scheduler allocation and 1,520
bytes across 100 calls, with no reallocations, numerical workspace, or field
clones. Four-worker and default-host Rust are operationally close to optimized
serial Fortran. Per the project stopping rule, explicit SIMD and further side
specialization are deferred unless the integrated boundary and halo driver
shows this kernel is material.

Reproduce with:

```sh
./scripts/benchmark-specified-boundary-relaxation-fortran.sh
cargo bench -p wrf-dynamics --bench specified_boundary_relaxation -- --noplot
```
