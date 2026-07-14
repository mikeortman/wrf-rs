# Periodic big-step column-mass CPU baseline — 2026-07-13

This matched release-mode baseline compares the Rust port of WRF `calc_mu_uv`
with the exact routine extracted from pinned WRF v4.7.1. Both horizontal axes
are periodic.

## Environment and optimization

- Apple M3 Max, 16 CPU cores, arm64 macOS
- rustc/cargo 1.96.0, LLVM 22.1.2
- GNU Fortran 16.1.0
- Rust bench profile: optimization level 3, ThinLTO, one codegen unit
- Fortran: `-O3 -flto`
- no fast-math and no explicit native-CPU flag on either side

These are comparable highest-normal optimization tiers, not identical compiler
behavior. The benchmark intentionally uses the repository's portable release
configuration.

## Matched workload

The physical mass domain is 1,024 × 1,024 points with one lower halo and one
stored upper momentum point on each axis. Both implementations use the same
single-precision initialization, separate perturbation/base fields, exact WRF
addition order, doubly periodic endpoints, and 2,099,200 output values per
call. Inputs and output storage are created before timing.

Commands:

```sh
cargo bench -p wrf-dynamics --bench column_mass_staggering -- \
  big_step_periodic_xy --noplot
./scripts/benchmark-periodic-column-mass-fortran.sh
```

## Rust results

Criterion central estimates and 95% confidence intervals:

| Workers | Time | Output throughput | Speedup vs. 1 worker |
|---:|---:|---:|---:|
| 1 | 359.64 µs `[353.56, 365.71]` | 5.8369 Goutput/s | 1.00× |
| 4 | 181.10 µs `[178.00, 184.29]` | 11.592 Goutput/s | 1.99× |
| 16 | 400.40 µs `[391.47, 409.82]` | 5.2428 Goutput/s | 0.90× |

Four workers are best. This low-arithmetic-intensity streaming kernel reaches
memory and scheduling limits before all heterogeneous host cores help.

## Matched optimized Fortran

The Fortran harness performs 100 excluded warm-up calls, then eleven samples of
500 calls:

```text
0.303078  0.293724  0.323382  0.394694  0.412366  0.361800
0.329830  0.314930  0.347120  0.360572  0.367454
```

The median is 347.120 µs and the observed range is
`[293.724, 412.366]` µs. One-worker Rust is 3.6% slower than serial Fortran;
four-worker Rust is 1.92× faster. The upstream routine contains no OpenMP
directives.

The serial implementations are close enough that SIMD adapters or a less
readable traversal are not justified without whole-model profiling. The
existing scalar row kernel is retained.

## Allocation evidence

The release allocation harness performs 100 warm-up calls, then measures two
100-call phases. At 1, 4, and 16 workers, every periodic phase records three
allocations totaling 4,560 bytes and zero reallocations. This matches the
non-periodic path and represents persistent scheduler queue traffic; the
numerical kernel allocates no field-sized or per-row scratch.

## Scope

This is one isolated ARW utility routine on one machine. It does not establish
whole-model performance or a forecast throughput improvement.
