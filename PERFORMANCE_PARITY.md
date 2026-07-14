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
- Stop tuning when Rust is in the same practical performance class, the
  standard multithreaded path is competitive, memory behavior is bounded, and
  no end-to-end profile identifies a material hotspot. A small benchmark gap
  does not justify complex SIMD, target-specific flags, or less readable code.

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

The more aggressive bench-only combination below was also measured without
changing production release compilation:

```sh
CARGO_PROFILE_BENCH_LTO=fat \
CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 \
RUSTFLAGS="-C target-cpu=native" \
cargo bench -p wrf-dynamics --bench <benchmark> -- --noplot
```

It did not improve the representative kernels consistently: serial
Held-Suarez improved 1.9% but its threaded cases regressed, while
positive-definite was effectively flat at one and four workers and improved
only its noisier 16-worker cases. Native CPU compilation also makes benchmark
binaries machine-specific. The portable ThinLTO profile therefore remains the
recorded default; deployment-specific tuning stays an explicit opt-in screen.

## Current results

| Kernel | Work per call | WRF Fortran | Rust, 1 worker | Best measured Rust | Status |
|---|---:|---:|---:|---:|---|
| Positive-definite sheet | 1,048,576 values | 1.709219 ms median `[1.676438, 1.775156]` | 1.1569 ms `[1.1517, 1.1632]` | 0.27485 ms (16 workers) | Rust serial 1.48× faster; Rust 16-worker 6.22× faster |
| Positive-definite slab | 1,048,576 values | 2.336656 ms median `[2.322000, 2.371812]` | 1.8084 ms `[1.7985, 1.8189]` | 0.34798 ms (16 workers) | Rust serial 1.29× faster; Rust 16-worker 6.71× faster |
| Held-Suarez damping | 2,097,152 momentum updates | 0.859712 ms median `[0.851224, 0.877004]` | 0.93459 ms `[0.92879, 0.94090]` | 0.29105 ms (4 workers) | Rust serial 8.7% slower; Rust 4-worker 2.95× faster |
| Column-mass staggering | 2,099,200 momentum-mass outputs | 0.286850 ms median `[0.284748, 0.309500]` | 0.33280 ms `[0.32970, 0.33632]` | 0.11532 ms (4 workers) | Rust serial 16.0% slower; Rust 4-worker 2.49× faster |

Domain topology is setup work and is not benchmarked as a timestep kernel.
Halo throughput is also not assigned a Rust/Fortran ratio yet: a four-rank
loopback result would mostly measure the local MPI runtime, while WRF aggregates
many fields through generated communication descriptors that are not ported.
The accepted implementation evidence is therefore bounded boundary-only
buffers, one patch allocation per MPI rank, non-blocking receives-before-sends,
and exact output parity. A matched communication benchmark becomes meaningful
after multi-field aggregation lands.

## Reproduction

```sh
cargo bench -p wrf-dynamics --bench positive_definite -- --noplot
cargo bench -p wrf-dynamics --bench held_suarez -- --noplot
cargo bench -p wrf-dynamics --bench column_mass_staggering -- --noplot
./scripts/benchmark-positive-definite-fortran.sh
./scripts/benchmark-held-suarez-fortran.sh
./scripts/benchmark-column-mass-staggering-fortran.sh
```

The Rust detailed results live under `docs/performance/`. Fortran drivers live
beside their parity fixtures so workload mapping can be reviewed with the
scientific oracle.

## Positive-definite comparison notes

- Date/machine/toolchains match the Held-Suarez comparison below.
- After 100 excluded warm-up calls, Fortran uses 11 samples of 32 calls. One
  field is restored immediately before every individually timed call, so setup
  is excluded without prewarming 32 separate fields or measuring an
  already-corrected early-exit path.
- Both sheet and slab contain 4,096 west-east lines of length 256. Every line
  contains a negative value and executes the correction path.
- The serial advantage is a combined routine result. It does not separately
  attribute savings from in-place mutation, removal of scratch allocation and
  copies, removal of the global negativity scan, or LLVM/GCC code generation.

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

## Column-mass staggering comparison notes

- Date, machine, and toolchains match the other 2026-07-13 comparisons.
- Both implementations use a 1,024 × 1,024 physical mass domain with storage
  for halos and upper stagger points. Both lower and upper physical boundaries
  execute on both axes.
- Fortran uses eleven samples of 500 calls after 100 warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark.
- All active outputs are overwritten on every call, so no output restoration is
  needed or timed. The four allocated fields are reused.
- Safe explicit-SIMD and iterator/autovectorization prototypes preserved parity
  but failed the representative performance gate and were removed.
