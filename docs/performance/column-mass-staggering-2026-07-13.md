# Column-mass staggering CPU baseline — 2026-07-13

This is the first matched release-mode baseline for the WRF
`calc_mu_staggered` port. It compares the parity-preserving scalar Rust
implementation with the exact extracted pinned Fortran routine.

## Environment

- Apple M3 Max, 16 CPU cores (12 performance, 4 efficiency), 128 GB memory
- macOS 26.2, arm64
- rustc/cargo 1.96.0, LLVM 22.1.2
- GNU Fortran 14.2.0
- workspace release/bench profile: optimization level 3, thin LTO, one codegen
  unit
- Criterion 0.7.0, 100 samples, default warm-up and statistical analysis

Rust and Fortran use comparable highest-normal optimization tiers. Rust uses
LLVM ThinLTO; Fortran uses `-O3 -flto`. Neither enables fast-math or explicitly
requests the local CPU. These settings are comparable, not identical compiler
behavior.

## Matched workload

The physical mass domain is 1,024 × 1,024 points. Allocated storage is
1,026 × 1,026 so each axis has one lower halo point and one stored upper
momentum point. The tile touches both physical boundaries on both axes.

Each call writes:

```text
(1,024 + 1) × 1,024 west-east momentum values
+ 1,024 × (1,024 + 1) south-north momentum values
= 2,099,200 outputs
```

Both implementations use the same `f32` initialization, dimensions, domain
bounds, tile bounds, boundary-copy paths, and source expression order. Inputs
remain immutable and every active output is overwritten, so neither benchmark
constructs nor restores fields in its timed interval.

Commands:

```sh
cargo bench -p wrf-dynamics --bench column_mass_staggering -- --noplot
./scripts/benchmark-column-mass-staggering-fortran.sh
```

## Rust results

Central estimates and 95% confidence intervals:

| Workers | Time | Output throughput | Speedup vs. 1 worker |
|---:|---:|---:|---:|
| 1 | 332.80 µs `[329.70, 336.32]` | 6.3078 Goutput/s | 1.00× |
| 4 | 115.32 µs `[114.38, 116.33]` | 18.203 Goutput/s | 2.89× |
| 16 | 242.03 µs `[239.79, 244.36]` | 8.6733 Goutput/s | 1.38× |

Four workers are best on this machine. The kernel streams four input loads and
two output stores across two passes, so memory bandwidth, scheduler overhead,
and the four efficiency cores make all-host execution slower than four workers.
This isolated result does not change the default policy of using host
parallelism; worker-count tuning remains a full-model integration concern.

## Matched optimized Fortran

The benchmark script extracts the exact `calc_mu_staggered` body from pinned
`dyn_em/module_big_step_utilities_em.F` and compiles it together with the
driver using GNU Fortran 14.2.0 and `-O3 -flto`. After 100 excluded warm-up
calls, eleven samples of 500 calls produced these milliseconds-per-call values:

```text
0.309500  0.300126  0.286678  0.286984  0.286112  0.286850
0.296442  0.297928  0.285546  0.284748  0.285340
```

The median is 286.850 µs and the observed range is
`[284.748, 309.500]` µs. One-worker Rust is 16.0% slower than serial Fortran.
Four-worker Rust is 2.49× faster, and 16-worker Rust is 1.19× faster, than the
serial upstream routine. The upstream routine contains no OpenMP directives.

These ratios describe one isolated routine and workload. They do not imply a
whole-model WRF speedup.

## Allocation measurement

The release allocation example creates the pool and four fields, performs 100
warm-up dispatches, then measures two consecutive phases of 100 calls:

```sh
cargo run -p wrf-dynamics --release \
  --example measure_column_mass_staggering_allocations
```

Every phase at 1, 4, and 16 workers recorded three allocations totaling 4,560
bytes and zero reallocations. That is 0.03 allocation and 45.6 bytes per call,
independent of field size and worker count. The allocations are persistent
Rayon/crossbeam dispatch-queue traffic; the numerical kernel allocates no
field-sized or per-row scratch.

## Generated code and rejected SIMD

GNU Fortran's `-fopt-info-vec-optimized` report confirms 128-bit and 64-bit
vectorization in the averaging loops. Rust release assembly for the retained
implementation uses scalar `fadd` and `fmul` instructions in those loops. That
combination, plus the serial gap, justified a safe-SIMD screen.

A `pulp` 0.22.3 prototype dispatched once per kernel and performed the four
ordered additions and final multiplication in SIMD lanes. Scalar/runtime SIMD
tests over every line length from 1 through 257 matched raw bits, and the full
240-value Fortran oracle remained exact. Criterion's stored comparison reported
5.8% and 5.5% regressions at one and four workers. An apparent 4.0% improvement
at 16 workers was not stable: the restored scalar implementation later ran at
242.03 µs versus 255.09 µs for the prototype. The SIMD code was removed.

A second safe-Rust prototype presented five equal-length slices through nested
iterators to encourage LLVM auto-vectorization. It measured 356.49 µs with one
worker, 117.54 µs with four, and 242.95 µs with 16. Its serial regression was
clear and its threaded results did not establish a durable representative gain,
so it was also removed.

The likely issue is per-row slice/SIMD adapter overhead on a low-arithmetic-
intensity streaming kernel. A future experiment should begin from whole-tile
fusion or a measured execution-layout change, not reintroduce either rejected
line adapter.

## Caveats

The machine was not isolated, frequency-pinned, or restricted to performance
cores. Criterion reported 5–9% outliers, and repeated 16-worker runs were
especially sensitive to system state. These measurements are a local
optimization baseline, not a cross-machine forecast.
