# Rust and Fortran performance tracking

This ledger compares each translated Rust numerical slice with the pinned WRF
Fortran routine on the same machine and matched workload. It complements the
detailed statistical benchmark pages; it is not a claim that isolated kernel
speedups predict whole-model speedup.

## Comparison policy

- Use the same precision, dimensions, active bounds, field initialization, and
  counted scientific operations.
- Compile the exact pinned WRF source, not a rewritten benchmark copy.
- Exclude fixture construction and output restoration from timed intervals.
- Do not enable fast-math, floating-point reassociation, or target-specific CPU
  flags unless the equivalent Rust configuration is separately measured and
  parity-tested.
- Record compiler versions, flags, worker counts, confidence intervals or raw
  repeated samples, and the date/machine.
- Compare serial Fortran with one-worker Rust. Report multithreaded Rust
  separately because these upstream routines contain no OpenMP directives.
- Treat differences below ordinary benchmark noise as parity, not a win.

## Optimization-level correspondence

The comparison matches production-style optimization tiers, not identical
compiler flags. The verified Rust bench invocation uses `-C opt-level=3`,
`-C lto=thin`, and `-C codegen-units=1`. The Fortran driver uses `-O3 -flto`.
Neither enables fast-math or explicitly requests the local CPU. On this Apple
AArch64 installation, GNU Fortran 14.2.0 reports `apple-m1` as its default CPU;
Rust enables the Apple AArch64 target features, including NEON, and `pulp`
performs runtime feature selection.

LLVM ThinLTO and GCC LTO are not equivalent algorithms, and Rust retains safe
slice/panic semantics that Fortran does not have. GCC's full `-flto` is arguably
the more aggressive interprocedural setting for this small driver. Results are
therefore labeled **matched workload, comparable optimization**, never
identical flags or identical compiler behavior.

## Current results

| Kernel | Work per call | WRF Fortran | Rust, 1 worker | Best measured Rust | Status |
|---|---:|---:|---:|---:|---|
| Positive-definite sheet/slab | 1,048,576 values | Pending matched run | Sheet 1.1569 ms; slab 1.8084 ms | Sheet 274.85 µs; slab 347.98 µs (16 workers) | Fortran baseline pending |
| Held-Suarez damping | 2,097,152 momentum updates | 0.859712 ms median `[0.851224, 0.877004]` | 0.93459 ms `[0.92879, 0.94090]` | 0.29105 ms (4 workers) | Rust serial 8.7% slower; Rust 4-worker 2.95× faster |

## Reproduction

```sh
cargo bench -p wrf-dynamics --bench positive_definite -- --noplot
cargo bench -p wrf-dynamics --bench held_suarez -- --noplot
./scripts/benchmark-held-suarez-fortran.sh
```

The Rust detailed results live under `docs/performance/`. Fortran drivers live
beside their parity fixtures so workload mapping can be reviewed with the
scientific oracle.

## Held-Suarez comparison notes

- Date/machine: 2026-07-13, Apple M3 Max, macOS 26.2 arm64.
- Fortran: GNU Fortran 14.2.0, `-O3 -flto`, seven samples of 500 calls after
  100 warm-up calls.
- Rust: rustc 1.96.0, workspace thin LTO/one codegen unit, Criterion 0.7 with
  100 statistical samples; verified `opt-level=3` bench profile.
- The Fortran routine is serial. Rust worker counts are explicit persistent
  pool sizes; output restoration occurs outside Criterion's measured interval.
- Both implementations process the same 256 × 64 × 64 active region for two
  staggered momentum components, or 2,097,152 updates per call.
- Rust uses accepted runtime-dispatched SIMD after exact scalar/SIMD tests over
  lengths 1–257; its pre-SIMD scalar baseline was 0.97808 ms with one worker.
