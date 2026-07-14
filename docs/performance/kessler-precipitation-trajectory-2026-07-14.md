# Kessler precipitation-trajectory performance

## Environment and matched workload

- Date: 2026-07-14
- Machine: Apple M3 Max, 12 performance plus 4 efficiency cores, 128 GB unified memory
- Operating system: macOS 26.2 arm64; no affinity or NUMA policy
- Rust: rustc 1.96.0 with LLVM 22.1.2, optimization level 3, ThinLTO, one codegen unit
- Fortran: GNU Fortran 16.1.0, `-O3 -flto -ffp-contract=off`
- Fast math and machine-specific target flags: disabled in both implementations
- Domain: 128 × 128 horizontal columns × 40 mass levels, plus 41 W levels
- Work: three consecutive 60-second warm-rain steps, or 1,966,080 point-steps per call

Each measured call begins from identical perturbation theta, `qv`/`qc`/`qr`,
inverse density, pressure, geopotential, and precipitation fields. It executes
the dependency-closed Kessler preparation, the pinned Kessler scheme, and
diabatic-tendency finalization three times with `use_theta_m` enabled,
microphysics heating enabled, and a 10 K/s tendency limit. Registry parsing,
scalar-layout resolution, workspace allocation, and fixture restoration are
outside both timers. Rust retains its public structural preflight inside every
timed step; the Fortran projection has no equivalent public validation layer.

The serial baseline is a clearly scoped Fortran Kessler-trajectory projection,
not the complete `moist_physics_prep_em`. It uses 40-level contiguous mass
fields and a separate 41-level W field, matching Rust's storage, and directly
compiles pinned WRF Kessler. Its preparation and finish loops contain exactly
the Kessler-relevant expressions used by Rust. This fuses WRF's independent
preparation passes in the same way as Rust while preserving each expression's
single-precision operation order. Pressure-at-W interpolation and logarithmic
top extrapolation are outside both timers because Kessler consumes neither.

## Numerical acceptance

The direct executable oracle extracts the pinned WRF v4.7.1 preparation and
finish routines from live source and compiles the pinned Kessler module. It
compares 35,280 raw-bit stage and checkpoint values through the direct reordered
species layout across active, heating-disabled, checkpoint-split, and IEEE
exceptional-sentinel cases. Exceptional inputs are placed in inactive storage
and preserved raw-bit exactly; active non-finite Kessler evolution is excluded
because WRF `MIN`/`MAX` propagation differs across GNU Fortran versions. Zero
comparisons are class-normalized. Separate Rust tests cover the canonical
layout. One- and four-worker Rust runs are
deterministic, and checkpoint-split execution matches uninterrupted state.

## Timing

| Implementation | Median per three-step trajectory | Relative to serial Fortran |
|---|---:|---:|
| Fortran Kessler-trajectory projection, serial | 152.494 ms | 1.00× |
| Rust, one worker | 139.851 ms | 1.09× faster |
| Rust, four workers | 76.002 ms | 2.01× faster |
| Rust, 16 workers | 60.174 ms | 2.53× faster |

Fortran's 31 samples span 150.568–163.757 ms, with p90 160.106 ms and
p99 163.242 ms. Criterion median confidence intervals are
138.876–140.682 ms, 75.298–77.036 ms, and 59.890–60.376 ms for one, four,
and 16 workers. The corresponding observed p90 values are 144.964 ms,
182.391 ms, and 62.956 ms. The four-worker samples contain severe tail
outliers, so its median speedup should not be read as a latency bound.

A bounds-checked Fortran build completes the identical three-step workload
without a runtime error. The optimized projection reports 1,966,080
point-steps and the stable anti-elision checksum
`-6.3547685321093565E+06`. GNU Fortran's optimization report confirms that
the projection's finish loop is vectorized with 16-byte vectors.

Ten fixture resets allocate nothing. Ten warmed three-step dispatches at every
worker count allocate once for 1,520 bytes and perform no reallocation; the
fixed batch-level allocation is persistent Rayon/crossbeam scheduler
bookkeeping, matching the accepted isolated-kernel behavior. All numerical
fields and Kessler scratch are allocated during setup and reused. The coupled
one-worker result is within nine percent of the matched serial projection, so
this receipt supports keeping the current scalar arithmetic and profiling the
future model runner before considering SIMD or additional cross-stage fusion.

## Reproduce

```sh
./scripts/run-kessler-precipitation-trajectory-oracle.sh
python3 tools/tracking.py run-benchmark \
  --id kessler-precipitation-trajectory \
  --output-directory benchmark-results
cargo run --release -p wrf-physics \
  --example measure_kessler_precipitation_trajectory_allocations
```
