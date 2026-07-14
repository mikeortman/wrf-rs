# Specified-boundary finalization performance baseline

This compares the exact WRF v4.7.1 `spec_bdy_final` body with the safe Rust
boundary-finalization capability. The matched workload reconstructs 205,820
vertical-momentum points on a 256 × 256 × 41 full-level domain with a
five-point specified zone and eight stored boundary points.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust 1.96.0 uses
optimization level 3, ThinLTO, and one codegen unit. Neither implementation
enables fast-math or a native CPU target. Measurements ran on an Apple M3 Max
under macOS 26.2 arm64.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran | 0.157440 ms median | 1.00× |
| Rust, 1 worker | 0.30229 ms | 92.0% slower |
| Rust, 4 workers | 0.10483 ms | 1.50× faster |
| Rust, 16 workers | 0.11565 ms | 1.36× faster |

Fortran's eleven samples ranged from 0.153350 to 0.287200 ms. Rust Criterion
intervals were 0.30036–0.30439, 0.10444–0.10529, and 0.11494–0.11642 ms.

The warmed 16-worker path records one scheduler allocation and 1,520 bytes
across 100 calls, with no reallocations, numerical workspace, or field clones.
Four-worker Rust already exceeds the optimized serial source, so policy
specialization, custom scheduling, and explicit SIMD are deferred unless an
integrated boundary driver identifies this perimeter kernel as material.

Reproduce with:

```sh
./scripts/benchmark-specified-boundary-finalization-fortran.sh
cargo bench -p wrf-dynamics --bench specified_boundary_finalization -- --noplot
```
