# Complete dry boundary-tendency performance baseline

This compares WRF v4.7.1 `spec_bdy_dry` plus its exact `spec_bdytend`
dependency with the safe Rust orchestration capability. The matched nested
workload uses a 256 × 256 × 40 mass grid and assigns 1,019,860 points across U,
V, PH, T, MU, and W per call.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust 1.96.0 uses
optimization level 3, ThinLTO, and one codegen unit. Neither implementation
enables fast-math or a native CPU target. Measurements ran on an Apple M3 Max
under macOS 26.2 arm64.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran | 0.485370 ms median | 1.00× |
| Rust, 1 worker | 0.48471 ms | effectively tied |
| Rust, 4 workers | 0.18404 ms | 2.64× faster |
| Rust, 16 workers | 0.50138 ms | 3.3% slower |

Fortran's eleven samples ranged from 0.448310 to 0.504650 ms. Rust's Criterion
confidence intervals were 0.47792–0.49217 ms, 0.18250–0.18574 ms, and
0.49413–0.50997 ms for one, four, and sixteen workers.

Across 100 warmed calls, the default Rust path records nine allocations and
nine deallocations totaling 13,680 bytes, with no reallocations. These are
persistent-pool scheduling costs; the wrapper owns no numerical scratch,
performs no field clone, and only borrows backend-native outputs and boundary
arrays.

The Rust implementation deliberately composes the verified scalar capability
instead of fusing six copies into a larger specialized loop. Serial parity is
already a statistical tie, four workers provide a substantial gain, and the
default host path remains within 3.3% of optimized serial Fortran. Explicit
SIMD and scheduler fusion therefore stop under the project's close-enough rule.

Reproduce with:

```sh
./scripts/benchmark-dry-boundary-tendencies-fortran.sh
cargo bench -p wrf-dynamics --bench dry_boundary_tendencies -- --noplot
```
