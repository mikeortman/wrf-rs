# Full inverse-density performance baseline — 2026-07-14

## Decision

Keep the safe contiguous row implementation. Four-worker Rust is 1.74× faster
than matched optimized serial Fortran, warmed dispatch has no numerical scratch,
and this isolated kernel does not justify explicit SIMD or custom scheduling.

## Matched workload

Both implementations execute the exact `calc_alt` operation over a
256 × 256 × 40 mass grid: 2,621,440 outputs per call. Storage also contains one
halo/upper-stagger point on both sides of every axis. The tile includes each
upper stagger and both implementations clip it to the physical mass domain.

The allocated `alt`, `al`, and `alb` fields are reused. Initialization and
allocation are outside timing. Every call performs one single-precision
addition and one output write per active point.

## Toolchains and optimization

- Machine: Apple M3 Max, 16 logical CPU workers reported by the host.
- Rust: release/bench workspace profile with optimization, ThinLTO, and one
  codegen unit.
- Fortran: GNU Fortran 16.1.0 with `-O3 -flto`.
- Neither side uses fast-math or an explicit native-CPU flag.

The Fortran harness extracts the exact pinned WRF v4.7.1 routine. It records
eleven samples of 50 calls after 20 warm-up calls. Criterion records 100 Rust
samples after its normal warm-up.

## Results

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| GNU Fortran, serial | 0.210880 ms median, range `[0.206400, 0.223980]` | 1.00× |
| Rust, 1 worker | 0.32594 ms, 95% interval `[0.32076, 0.33097]` | 54.6% slower |
| Rust, 4 workers | 0.12102 ms, 95% interval `[0.11796, 0.12446]` | 1.74× faster |
| Rust, 16 workers | 0.39732 ms, 95% interval `[0.38931, 0.40643]` | 88.4% slower |

Four workers are best. Sixteen workers add scheduling and memory-system
contention to a kernel with very little arithmetic per byte. The default
backend still uses host parallelism; worker-count tuning remains a deployment
choice rather than a different scientific implementation.

## Allocation evidence

Each measurement runs 100 calls after 100 warm-up calls.

| Workers | First measured phase | Settled phase | Reallocations |
|---:|---:|---:|---:|
| 1 | 2 allocations, 3,040 bytes | 1 allocation, 1,520 bytes | 0 |
| 4 | 2 allocations, 3,040 bytes | 1 allocation, 1,520 bytes | 0 |
| 16 | 2 allocations, 3,040 bytes | 1 allocation, 1,520 bytes | 0 |

There is no field-sized or numerical scratch. The small allocation is the
existing Rayon/crossbeam scheduler behavior and is independent of domain size.

## SIMD decision

The hot loop uses equal-length contiguous slices and a simple scalar addition,
which already gives LLVM a normal autovectorization opportunity. Because the
standard four-worker path exceeds matched Fortran and whole-model profiling is
not yet available, a `pulp` specialization would add maintenance and testing
cost without a demonstrated model-level benefit.

## Reproduction

```sh
./scripts/benchmark-inverse-density-fortran.sh
cargo bench -p wrf-dynamics --bench inverse_density -- --noplot
cargo run -p wrf-dynamics --release --example measure_inverse_density_allocations
```
