# Held-Suarez CPU baseline — 2026-07-13

This is the first statistical release-mode baseline for the WRF
`held_suarez_damp` port. It measures the scalar, exact-bit implementation over
the persistent Rayon scheduler.

## Environment

- Apple M3 Max, 16 CPU cores (12 performance, 4 efficiency), 128 GB memory
- macOS 26.2, arm64
- rustc/cargo 1.96.0, LLVM 22.1.2
- workspace release profile: thin LTO, one codegen unit
- Criterion 0.7.0, 100 samples, default warm-up and statistical analysis

## Workload

The active region contains 256 west-east points, 64 vertical levels, and 64
south-north lines for each of the two momentum components. A dispatch therefore
performs 2,097,152 momentum-tendency updates. The allocated fields include one
halo point on each horizontal side and a pressure reference level below the
active vertical range.

All levels participate in the pressure calculation. The base-pressure profile
crosses the `sigma = 0.7` damping boundary, so the workload contains damped and
unchanged levels. Criterion restores only the two tendency outputs in excluded
`LargeInput` batch setup; the four immutable domain-sized inputs are reused.

Command:

```sh
cargo bench -p wrf-dynamics --bench held_suarez -- --noplot
```

## Results

Central estimates and 95% confidence intervals:

| Workers | Time | Momentum-update throughput | Speedup vs. 1 worker |
|---:|---:|---:|---:|
| 1 | 978.08 µs `[969.95, 986.03]` | 2.1442 Gupdate/s | 1.00× |
| 4 | 305.33 µs `[303.48, 307.13]` | 6.8684 Gupdate/s | 3.20× |
| 16 | 550.94 µs `[545.63, 556.36]` | 3.8065 Gupdate/s | 1.78× |

## Interpretation

Four workers provide strong scaling and are the best configuration on this
machine for this workload. Using all 16 heterogeneous cores is 80% slower than
four workers, although still faster than one. The kernel performs repeated
loads from four immutable fields and writes two outputs, so memory bandwidth,
cache pressure, scheduling granularity, and the machine's four efficiency cores
can outweigh additional parallelism.

The result does not justify changing the default backend policy by itself:
other WRF kernels have different arithmetic intensity, and a full timestep can
amortize scheduling differently. It does establish that worker-count tuning or
topology-aware execution will matter before production-scale CPU runs.

The contiguous west-east loop is a credible SIMD candidate because it performs
independent pressure arithmetic and tendency updates without a reduction.
Explicit SIMD still requires raw-bit differential tests across lane/tail
boundaries and a benchmark win at representative worker counts. No SIMD crate
is added by this baseline.

## Memory behavior

The field bundle borrows all six fields. Shape/range descriptors are small
values, and the numerical implementation creates no field-sized or per-line
scratch storage. Criterion's output clones are fixture restoration outside the
measured interval, not timestep behavior. The persistent scheduler's small
dispatch allocations have been measured for the positive-definite family but
have not yet been separately instrumented for this two-pass kernel.

## Matched WRF Fortran comparison

The pinned upstream routine was compiled with GNU Fortran 14.2.0 using
`-O3 -flto`, without fast-math or a native-CPU flag. Seven samples of 500 calls
on the identical initialized field and active bounds measured 0.851224–0.877004
ms per call, with a median of 0.859712 ms.

The current one-worker Rust kernel is therefore 13.8% slower than optimized
serial Fortran. Four-worker Rust is 2.82× faster than serial Fortran, and
16-worker Rust is 1.56× faster. This is an isolated routine comparison, not a
whole-model speedup. The serial gap is retained as an optimization target; the
cross-language policy and summary are in `PERFORMANCE_PARITY.md`.

## Caveats

The machine was not isolated or frequency-pinned. The 16-worker case reported
9% outliers and mixes performance and efficiency cores. These measurements are
a local optimization baseline, not a cross-machine WRF comparison.
