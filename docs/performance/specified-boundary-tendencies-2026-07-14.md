# Specified-boundary tendency-assignment performance baseline

This compares the exact WRF v4.7.1 `spec_bdytend` body with the safe Rust
boundary-tendency capability. The matched workload assigns 200,800 points on a
256 × 256 × 40 mass grid with a five-point specified zone and eight stored
boundary points.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust 1.96.0 uses
optimization level 3, ThinLTO, and one codegen unit. Neither implementation
enables fast-math or a native CPU target. Measurements ran on an Apple M3 Max
under macOS 26.2 arm64.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran | 0.082900 ms median | 1.00× |
| Rust, 1 worker | 0.069473 ms | 1.19× faster |
| Rust, 4 workers | 0.028529 ms | 2.91× faster |
| Rust, 16 workers | 0.085521 ms | 3.2% slower |

Fortran's eleven samples ranged from 0.079250 to 0.089420 ms. Rust Criterion
intervals were 0.069133–0.069811, 0.028038–0.029097, and
0.084710–0.086324 ms.

The warmed 16-worker path records one scheduler allocation and 1,520 bytes
across 100 calls, with no reallocations, numerical workspace, or field clones.
Serial and four-worker Rust already exceed optimized serial Fortran. The
default host pool is operationally equal for this thin perimeter copy, so
custom worker selection and explicit SIMD are deferred unless an integrated
boundary driver identifies material value.

Reproduce with:

```sh
./scripts/benchmark-specified-boundary-tendencies-fortran.sh
cargo bench -p wrf-dynamics --bench specified_boundary_tendencies -- --noplot
```
