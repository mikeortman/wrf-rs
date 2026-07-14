# Held-Suarez CPU baseline — 2026-07-13

This is the first statistical release-mode baseline for the WRF
`held_suarez_damp` port. The original scalar baseline and accepted safe-SIMD
implementation both run over the persistent Rayon scheduler.

## Environment

- Apple M3 Max, 16 CPU cores (12 performance, 4 efficiency), 128 GB memory
- macOS 26.2, arm64
- rustc/cargo 1.96.0, LLVM 22.1.2
- workspace release profile: thin LTO, one codegen unit
- Criterion 0.7.0, 100 samples, default warm-up and statistical analysis

The actual Rust bench invocation was verified as
`-C opt-level=3 -C lto=thin -C codegen-units=1`. GNU Fortran uses `-O3 -flto`;
its compiler reports `apple-m1` as the default CPU on this host. Neither side enables fast-math or an
explicit native-CPU flag. These are comparable highest-normal optimization
tiers, not identical compiler settings: GCC LTO and LLVM ThinLTO differ, and
safe Rust retains language semantics absent from Fortran.

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
| 1 | 934.59 µs `[928.79, 940.90]` | 2.2439 Gupdate/s | 1.00× |
| 4 | 291.05 µs `[288.32, 293.92]` | 7.2056 Gupdate/s | 3.21× |
| 16 | 521.22 µs `[516.99, 526.23]` | 4.0236 Gupdate/s | 1.79× |

## Interpretation

Four workers provide strong scaling and are the best configuration on this
machine for this workload. Using all 16 heterogeneous cores is 79% slower than
four workers, although still faster than one. The kernel performs repeated
loads from four immutable fields and writes two outputs, so memory bandwidth,
cache pressure, scheduling granularity, and the machine's four efficiency cores
can outweigh additional parallelism.

The result does not justify changing the default backend policy by itself:
other WRF kernels have different arithmetic intensity, and a full timestep can
amortize scheduling differently. It does establish that worker-count tuning or
topology-aware execution will matter before production-scale CPU runs.

## Accepted SIMD optimization

GNU Fortran's `-fopt-info-vec-optimized` report confirms that both upstream
west-east loops are vectorized with 128-bit and 64-bit vectors. The first Rust
scalar pre-slicing experiment regressed one- and four-worker performance by
2.6–2.8% and was removed. A dispatch-only `pulp` experiment produced a small
gain but did not explicitly control vector operations.

The accepted `pulp` 0.22.3 path dispatches once per complete kernel, then runs
manual SIMD arithmetic inside each Rayon-owned west-east line. The scalar tail
uses the original formula. Tests compare raw bits between `pulp::Scalar` and
runtime SIMD for every length from 1 through 257, covering vector boundaries,
unaligned staggered slices, and tails; the full Fortran fixture also remains
exact.

Compared with the retained scalar baseline, SIMD improves one worker by 4.4%,
four workers by 4.7%, and 16 workers by 5.4%. The implementation is isolated in
the `held_suarez::simd` submodule, uses no local unsafe code, and adds no
concurrency layer.

## Memory behavior

The field bundle borrows all six fields. Shape/range descriptors are small
values, and the numerical implementation creates no field-sized or per-line
scratch storage. Criterion's output clones are fixture restoration outside the
measured interval, not timestep behavior. The persistent scheduler's small
dispatch allocations were separately instrumented after warm-up. Each
100-dispatch phase at 1, 4, and 16 workers used exactly three 1,520-byte
allocations (4,560 bytes total), with no reallocations. That is an amortized
0.03 allocation and 45.6 bytes per call, independent of field size and worker
count; no numerical scratch or field clone occurs.

## Matched WRF Fortran comparison

The pinned upstream routine was compiled with GNU Fortran 14.2.0 using
`-O3 -flto`, without fast-math or a native-CPU flag. Seven samples of 500 calls
on the identical initialized field and active bounds measured 0.851224–0.877004
ms per call, with a median of 0.859712 ms.

The optimized one-worker Rust kernel is therefore 8.7% slower than optimized
serial Fortran. Four-worker Rust is 2.95× faster than serial Fortran, and
16-worker Rust is 1.65× faster. This is an isolated routine comparison, not a
whole-model speedup. The remaining serial gap is retained as an optimization
target; the cross-language policy and summary are in `PERFORMANCE_PARITY.md`.

## Caveats

The machine was not isolated or frequency-pinned. The 16-worker case reported
9% outliers and mixes performance and efficiency cores. These measurements are
a local optimization baseline, not a cross-machine WRF comparison.

## Rejected bench-only compiler profiles

Bench-only fat LTO and `target-cpu=native` were tested without changing the
production release profile. The combined native/fat build received a full
100-sample run; the separated knobs received quick screens.

| Bench compilation | 1 worker | 4 workers | 16 workers | Decision |
|---|---:|---:|---:|---|
| Portable ThinLTO baseline | 934.59 µs | 291.05 µs | 521.22 µs | Retained |
| Native CPU + fat LTO | 917.09 µs | 293.71 µs | 540.73 µs | Rejected: 1.9% serial gain, 4/16 regressions |
| Native CPU + ThinLTO, quick | 936.23 µs | 311.49 µs | 517.57 µs | Rejected at screen |
| Generic CPU + fat LTO, quick | 926.78 µs | 319.74 µs | 525.39 µs | Rejected at screen |

No variant improves the representative worker counts consistently, and native
compilation reduces binary portability. The workspace keeps portable ThinLTO;
these experiments can be repeated per deployment target if a full-model profile
later shows a meaningful opportunity.
