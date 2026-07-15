# Registry-backed ARW accepted-stage trajectory performance protocol

## Status and scope

This page records the local matched-measurement receipt for issue #82. The
Rust runner, direct parity comparison, allocation check, and matched Fortran
benchmark passed before these values were recorded. This receipt applies only
to the dependency-closed projection below; it is not whole-model evidence.

The measured work is a dependency-closed accepted-stage projection. It binds
ordinary and moist fields selected from the WRF Registry into typed Rust model
state, then executes the already accepted preparation, first dry Runge-Kutta
tendency, three-substep acoustic trajectory, acoustic finalization, and Kessler
preparation-driver-finish stages. The matched Fortran projection invokes the
pinned WRF v4.7.1 routine bodies in the same order.

This is deliberately not a complete `solve_em` timestep. It excludes unported
dynamics tendencies, scalar advection, halo and boundary communication, and
the other stages between these selected routines in WRF's Runge-Kutta loop.
The result therefore does not advance whole-model parity or the end-to-end
idealized-case gate.

## Numerical gate before measurement

The direct oracle passed before either implementation was timed. It:

- verifies the pinned WRF source checksums;
- compare raw single-precision bits for complete allocated storage at every
  exposed accepted-stage checkpoint and report the first divergence;
- proved that inactive storage and Registry-resolved moisture ordering are
  preserved;
- covered malformed bindings and failure atomicity;
- matched a recreated nonrestart workspace supplied with identical projected
  stage inputs; and
- matched one-worker and four-worker Rust execution exactly.

The executable comparison matched 18,324 shared raw `f32` values across seven
exposed stage boundaries. The live pinned-WRF run retains 63,468 internal
checkpoint values, so a failure reports the first differing stage, field, and
flat index. Rust's compact surface-based Kessler adapter is a storage-boundary
difference; the emitter maps it back to WRF's common padded oracle allocation.

The six-point-per-axis oracle fixture is a correctness case, not the
performance workload.

## Environment and workload

- Date: 2026-07-14
- Machine: Apple M3 Max, 12 performance plus 4 efficiency cores, 128 GB unified memory
- Operating system: macOS 26.2 arm64; no affinity or NUMA policy
- Rust: rustc 1.96.0, LLVM 22.1.2, optimization level 3, ThinLTO, one codegen unit
- Fortran: GNU Fortran 16.1.0, `-O3 -flto -ffp-contract=off`
- Machine-specific target flags and fast math: disabled for both implementations
- Common storage: 64 × 64 × 40; active mass work: 62 × 62 × 38
- Work per sample: one continuous accepted-stage trajectory with three acoustic substeps
- Samples: 21 Fortran samples; Criterion sample size 300 with a 10-second measurement window

Registry parsing, binding, allocation, and deterministic reset were outside
both timers. Rust retained public preflight, its padded-to-compact
microphysics adapters, and safe worker dispatch inside the timer. Fortran
called the pinned routine bodies directly with the identical scientific
controls and stage order.

## Timing

| Implementation | Median per trajectory | Variability | Relative to serial Fortran |
|---|---:|---:|---:|
| Fortran, serial | 12.746 ms | 12.503–13.743 ms; p90 13.100 ms | 1.00× |
| Rust, one worker | 44.378 ms | 95% CI 44.290–44.551 ms; p90 45.782 ms | 3.48× slower |
| Rust, four workers | 16.526 ms | 95% CI 16.488–16.580 ms; p90 17.085 ms | 1.30× slower |

The Fortran anti-elision checksum was
`1.4192720573265105E+03`. Numerical error on the direct correctness workload
was zero shared bits, and one/four-worker Rust results were bitwise identical.
The composed Rust path does not yet reach the serial Fortran baseline even
with four workers. The primary remaining performance question is not yet
isolated to a kernel: follow-up profiling must separate runner preflight,
adapter copies, scheduler overhead, and the accepted component kernels before
any low-level optimization is justified.

Five warmed trajectories at each Rust worker count performed 9 bounded
scheduler allocations, 0 reallocations, and 13,680 allocated bytes in total;
reset performed no allocation. No model-sized field was allocated or cloned
inside the trajectory.

## Matched benchmark boundary

The `registry-backed-arw-trajectory` catalog suite owns only this composed
runner. Registry parsing, Registry-to-state binding validation, field and
workspace allocation, deterministic fixture construction, and fixture reset
remain outside both timers. Each timed call executes the same accepted stages
from the same initial values. Public Rust preflight remains inside the Rust
timer and must not be silently removed from the Fortran comparison narrative.

The Rust benchmark must report at least one-worker and four-worker cases and
must reuse model state and scratch storage without allocating or cloning
model-sized fields in the timed loop. The Fortran build must use release-like
optimization with floating-point contraction disabled unless both sides adopt
and document another equivalent policy. Fast math and machine-specific target
flags must not be enabled silently.

Measurement must use warm-up iterations and multiple samples and report
medians plus variability, not a single best run. The receipt must distinguish
the serial Fortran baseline from each Rust worker count and record any
operation-order or reproducibility difference. Allocation measurement is a
separate required check and does not belong in the kernel timer.

## Reproduce

```sh
./scripts/run-registry-backed-arw-trajectory-oracle.sh
python3 tools/tracking.py run-benchmark \
  --id registry-backed-arw-trajectory \
  --output-directory benchmark-results
cargo run --release -p wrf-model \
  --example measure_registry_backed_arw_trajectory_allocations
```

The performance workflow routes changes in the `wrf-model` runner, its
accepted acoustic finalization dependency, the matched benchmark and oracle
fixtures, and their scripts to this suite only. Workspace bootstrap changes
are scoped to this suite when they add only the `wrf-model` package and its
lockfile stanza.
