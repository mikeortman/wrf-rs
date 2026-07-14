# Complete acoustic boundary-stage performance

Date: 2026-07-14. Machine: Apple M3 Max with 12 performance and 4 efficiency
cores, 128 GB unified memory, macOS 26.2 arm64. Toolchains: Rust 1.96.0 with
LLVM 22.1.2 and GNU Fortran 16.1.0. No affinity or NUMA policy was applied.

## Matched workload

Both drivers execute the exact nonhydrostatic specified-boundary call sequence
from `small_step_prep` through three acoustic substeps on a 128 × 128 × 40 mass
grid. Storage includes WRF's four-point horizontal zones and the upper full
level. Each measured call starts from the same initialized state; reset work is
outside both timers. The timed region includes the complete structural
preflight on Rust because validation is part of its public capability.

Fortran uses `-O3 -flto -ffp-contract=off` and records 31 one-call samples. Rust
uses optimization level 3, ThinLTO, one codegen unit, and Criterion backends
configured with 1, 4, and 16 workers. Neither side enables fast-math,
target-specific code, explicit SIMD, or a changed reduction order.

## Numerical acceptance

The stage oracle extracts every participating routine from pinned WRF v4.7.1.
Its role-distinct, coordinate-dependent fixture covers full periodic,
specified, and nested domains, a west-edge partial tile, an interior
boundary-inactive tile, and an IEEE-injected specified case. It compares 149
complete affected-field snapshots per case after every numerical or boundary
insertion, plus all 13 complete final fields. Across all six cases this is
1,108,350 intermediate and 98,550 final values. Finite values, infinities, and
signed zeros have zero bit, absolute, and relative error; NaNs match by class.
The benchmark uses the same specified control path and coefficients at larger
dimensions.

## Results

| Implementation | Time per complete stage | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 23.282 ms median | 1.00× |
| Rust, one worker | 141.19 ms | 6.06× slower |
| Rust, four workers | 41.536 ms | 1.78× slower |
| Rust, 16 workers | 27.760 ms | 19.2% slower |

Fortran's 31 samples span 21.996–26.073 ms. Criterion intervals are
140.47–141.97 ms, 41.238–41.855 ms, and 27.354–28.208 ms. The 16-worker result
processes the benchmark's 8.52 million reported output elements at about
307 million elements per second.

Across 25 warmed dispatches at each worker count, the composed call records 43
scheduler allocations totaling 65,360 bytes, with no reallocations, numerical
scratch, or field clones. The implicit vertical solve uses its caller-owned
workspace. Resetting the fixture is excluded from timing and performs no
allocation.

The direct integrated measurement supersedes the earlier arithmetic trajectory
estimate for this boundary-bearing workload. It shows that the safe ordinary
16-worker path is close to, but does not yet equal, optimized serial WRF. No
cache-counter, assembly, or stage-timing profile was collected in this slice;
therefore cross-stage fusion, explicit SIMD, and worker specialization would be
speculative. The next performance slice should attribute time among numerical
dispatch, preflight, and boundary insertion before changing source order or
floating-point semantics.

## Reproduce

```sh
./scripts/benchmark-acoustic-boundary-stage-fortran.sh
cargo bench -p wrf-dynamics --bench acoustic_boundary_stage -- --noplot
cargo run -p wrf-dynamics --release --example measure_acoustic_boundary_stage_allocations
```
