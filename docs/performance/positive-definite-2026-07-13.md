# Positive-definite CPU baseline — 2026-07-13

This is the first statistical release-mode baseline for the WRF
positive-definite sheet and slab kernels. It measures the parity-preserving
scalar implementation on top of the persistent Rayon scheduler. The parent
foundation commit is `bb6cc55`.

## Environment

- Apple M3 Max, 16 CPU cores (12 performance, 4 efficiency), 128 GB memory
- macOS 26.2, arm64
- rustc/cargo 1.96.0, LLVM 22.1.2
- workspace release profile: thin LTO, one codegen unit
- Criterion 0.7.0, 100 samples, default warm-up and statistical analysis

Criterion 0.7 is intentional: the workspace declares Rust 1.85 compatibility,
while Criterion 0.8 requires Rust 1.86.

## Workload

Both variants process 1,048,576 `f32` values arranged as 4,096 contiguous
west-east lines of length 256. Every line contains one negative value and must
take the correction path. The slab is shaped as 64 bottom-top by 64
south-north lines. Criterion clones the input field in excluded `LargeInput`
batch setup so fixture restoration is not included in kernel time.

Command:

```sh
cargo bench -p wrf-dynamics --bench positive_definite -- --noplot
```

## Results

Central estimates and 95% confidence intervals:

| Kernel | Workers | Time | Throughput | Speedup vs. 1 worker |
|---|---:|---:|---:|---:|
| Sheet | 1 | 1.1569 ms `[1.1517, 1.1632]` | 906.33 Melem/s | 1.00× |
| Sheet | 4 | 316.00 µs `[315.48, 316.57]` | 3.3182 Gelem/s | 3.66× |
| Sheet | 16 | 274.85 µs `[271.77, 278.06]` | 3.8150 Gelem/s | 4.21× |
| Slab | 1 | 1.8084 ms `[1.7985, 1.8189]` | 579.84 Melem/s | 1.00× |
| Slab | 4 | 483.08 µs `[482.41, 483.81]` | 2.1706 Gelem/s | 3.74× |
| Slab | 16 | 347.98 µs `[342.64, 354.73]` | 3.0133 Gelem/s | 5.20× |

## Interpretation

Four workers capture most of the available sheet speedup. Sixteen workers add
about 15% sheet throughput over four, indicating that memory traffic,
per-line scalar reductions, and scheduler overhead dominate before all cores
scale linearly. The slab retains more benefit at 16 workers but still shows
diminishing returns.

Release assembly on this AArch64 target uses scalar `fsub`, `fmul`, `fadd`, and
`fminnm` instructions in the line loops; no packed NEON arithmetic was emitted.
The pointwise translation and two-step scaling loops are therefore credible
explicit-SIMD candidates. Minimum and sum reductions remain ordered scalar
loops because reassociation would change parity.

## Allocation measurement

`stats_alloc` wraps the system allocator in a release example without local
unsafe code. The persistent pool and 1,048,576-value fields are created first;
100 dispatches warm each kernel; then two consecutive 100-dispatch phases are
measured. Fixture restoration is an in-place slice copy.

```sh
cargo run -p wrf-dynamics --release \
  --example measure_positive_definite_allocations
```

Across 1, 4, and 16 workers, each 100-dispatch phase recorded either one
1,520-byte allocation or two totaling 3,040 bytes, with zero reallocations.
This is amortized Rayon/crossbeam injection-queue storage: it is independent of
field size and worker count, and it persists even after workload warm-up. The
numerical kernels still allocate no field-sized scratch and copy no corrected
lines.

The result does **not** support a zero-allocation claim. It supports a much more
precise one: the measured scheduler cost is 0.01–0.02 small allocations and
15.2–30.4 bytes per million-point dispatch. A future CPU timestep execution
scope could batch kernel calls under one pool installation if profiles show
this amortized injection cost matters.

## Caveats

The machine was not isolated, frequency-pinned, or restricted to performance
cores. Criterion reported 2–11% outliers depending on the case. These results
are appropriate as a local optimization baseline, not as a cross-machine WRF
performance comparison.

## Matched WRF Fortran comparison

The pinned upstream module was compiled with GNU Fortran 14.2.0 using
`-O3 -flto`, without fast-math or an explicit native-CPU flag. After 100
excluded warm-up calls, eleven samples each contain 32 individually timed
calls. One field is restored immediately before every call, excluding fixture
restoration while preventing later calls from taking the already-corrected
early exit and avoiding a prewarmed field pool.

| Kernel | Fortran median and observed range | Rust, 1 worker | Serial ratio | Best Rust ratio |
|---|---:|---:|---:|---:|
| Sheet | 1.709219 ms `[1.676438, 1.775156]` | 1.1569 ms | Rust 1.48× faster | Rust 16-worker 6.22× faster |
| Slab | 2.336656 ms `[2.322000, 2.371812]` | 1.8084 ms | Rust 1.29× faster | Rust 16-worker 6.71× faster |

The workloads have identical 256 × 4,096 line geometry and initialization.
This benchmark measures the combined routine implementations; it does not
isolate scratch allocation/copies, the repeated negativity scan, or compiler
code generation. See `PERFORMANCE_PARITY.md` for the cross-language policy and
optimization-level caveats.

## Rejected bench-only compiler profile

The portable ThinLTO baseline was compared with an opt-in native CPU plus fat
LTO build. This affected only the Cargo bench profile for the command; it did
not change production release settings.

| Kernel | Workers | Portable ThinLTO | Native CPU + fat LTO | Change |
|---|---:|---:|---:|---:|
| Sheet | 1 | 1.1569 ms | 1.1616 ms | 0.4% slower |
| Sheet | 4 | 316.00 µs | 315.81 µs | effectively flat |
| Sheet | 16 | 274.85 µs | 268.07 µs | 2.5% faster |
| Slab | 1 | 1.8084 ms | 1.8147 ms | 0.3% slower |
| Slab | 4 | 483.08 µs | 484.87 µs | 0.4% slower |
| Slab | 16 | 347.98 µs | 323.93 µs | 6.9% faster |

The profile does not improve the representative one- and four-worker cases,
and its gains occur at the mixed performance/efficiency-core count with greater
scheduling noise. It is not adopted as the benchmark default. The exact opt-in
command is retained in `PERFORMANCE_PARITY.md` for deployment-specific repeats.

## Rejected `pulp` SIMD experiment

`pulp` 0.22.3 was prototyped with one runtime dispatch per kernel. Minimum and
sum reductions stayed scalar and ordered; only translation and the two ordered
multiplications used SIMD lanes. Temporary differential tests compared runtime
NEON with `pulp::Scalar` across line lengths 1–257, vector-width boundaries,
tails, and an unaligned slab subrange. Every value matched exactly by bits, and
the upstream Fortran fixtures continued to pass.

The full Criterion run did not justify keeping the added abstraction:

| Kernel | Workers | Scalar baseline | SIMD prototype | Change |
|---|---:|---:|---:|---:|
| Sheet | 1 | 1.1569 ms | 1.1741 ms | 1.5% slower |
| Sheet | 4 | 316.00 µs | 320.37 µs | 1.4% slower |
| Sheet | 16 | 274.85 µs | 268.76 µs | 2.2% faster |
| Slab | 1 | 1.8084 ms | 1.8761 ms | 3.7% slower |
| Slab | 4 | 483.08 µs | 500.29 µs | 3.6% slower |
| Slab | 16 | 347.98 µs | 325.50 µs | 6.5% faster |

The per-core and four-worker regressions outweigh gains seen only at 16 workers,
where scheduling, mixed performance/efficiency cores, and system noise have
more influence. The implementation and dependency were removed. This decision
is kernel-specific: `pulp` remains a strong candidate for longer pointwise
kernels with fewer ordered reduction passes.
