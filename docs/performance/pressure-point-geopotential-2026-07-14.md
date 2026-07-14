# Pressure-point geopotential performance baseline — 2026-07-14

## Decision

Keep the safe contiguous row implementation. One-worker Rust is 10.6% slower
than matched optimized Fortran, four-worker Rust is 2.86× faster, and warmed
dispatch uses no numerical scratch. Explicit SIMD and custom scheduling are not
justified for this isolated kernel.

## Matched workload

Both implementations execute the exact `calc_php` operation over a
256 × 256 × 40 mass grid: 2,621,440 outputs per call. Storage contains the
vertical full level above every active pressure point plus horizontal halos and
all upper stagger points. The tile includes each upper stagger; both routines
clip output to the physical mass domain.

The allocated `php`, `ph`, and `phb` fields are reused. Initialization and
allocation are outside timing. Every active output reads base-state and
perturbation geopotential at the current and upper full levels, preserves WRF's
four-term addition order, then multiplies by one half.

## Toolchains and optimization

- Machine: Apple M3 Max, 16 logical CPU workers reported by the host.
- Rust: optimized workspace bench profile with ThinLTO and one codegen unit.
- Fortran: GNU Fortran 16.1.0 with `-O3 -flto`.
- Neither side uses fast-math or an explicit native-CPU flag.

The Fortran harness extracts the exact pinned WRF v4.7.1 routine. It records
eleven samples of 50 calls after 20 warm-up calls. Criterion records 100 Rust
samples after its normal warm-up.

## Results

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| GNU Fortran, serial | 0.402140 ms median, range `[0.377740, 0.464480]` | 1.00× |
| Rust, 1 worker | 0.44482 ms, 95% interval `[0.44034, 0.44991]` | 10.6% slower |
| Rust, 4 workers | 0.14072 ms, 95% interval `[0.13797, 0.14386]` | 2.86× faster |
| Rust, 16 workers | 0.40852 ms, 95% interval `[0.39959, 0.41791]` | 1.6% slower |

Four workers are best. Sixteen workers add dispatch and memory-system
contention to a low-arithmetic-intensity kernel. Worker-count tuning remains a
deployment concern; it does not change the scientific implementation.

## Allocation evidence

Each phase measures 100 calls after 100 warm-up calls.

| Workers | First measured phase | Settled phase | Reallocations |
|---:|---:|---:|---:|
| 1 | 2 allocations, 3,040 bytes | 1 allocation, 1,520 bytes | 0 |
| 4 | 2 allocations, 3,040 bytes | 1 allocation, 1,520 bytes | 0 |
| 16 | 2 allocations, 3,040 bytes | 1 allocation, 1,520 bytes | 0 |

There is no field-sized or numerical scratch. The small allocation is the
existing Rayon/crossbeam scheduler behavior and is independent of domain size.

## SIMD decision

Equal-length west-east slices expose a conventional compiler-vectorizable loop
without unsafe code. A nested iterator prototype was about 6% faster but made
the four-input ownership relationship materially harder to read; the clear
index-based loop is retained because four-worker Rust remains 2.86× faster than
Fortran. The parity oracle also includes a source-order-sensitive overflow case,
so any explicit vector path would need to prove it does not reassociate the four
additions. That additional complexity has no current model-level justification.

## Reproduction

```sh
./scripts/benchmark-pressure-point-geopotential-fortran.sh
cargo bench -p wrf-dynamics --bench pressure_point_geopotential -- --noplot
cargo run -p wrf-dynamics --release --example measure_pressure_point_geopotential_allocations
```
