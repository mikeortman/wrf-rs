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

## Allocation status

Source inspection proves that the Rust kernels allocate no scratch line and do
not copy corrected lines. This does not yet prove zero allocations across the
entire Rayon dispatch path. A trustworthy steady-state allocator measurement
is still required before recording a zero-allocation claim. The benchmark's
input clone is outside the measured interval but intentionally allocates during
fixture setup.

## Caveats

The machine was not isolated, frequency-pinned, or restricted to performance
cores. Criterion reported 2–11% outliers depending on the case. These results
are appropriate as a local optimization baseline, not as a cross-machine WRF
performance comparison.
