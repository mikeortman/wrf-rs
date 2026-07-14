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
| Periodic big-step column mass | 2,099,200 momentum-mass outputs | 0.347120 ms median `[0.293724, 0.412366]` | 0.35964 ms `[0.35356, 0.36571]` | 0.18110 ms (4 workers) | Rust serial 3.6% slower; Rust 4-worker 1.92× faster; stop tuning |
| Momentum coupling | 7,950,336 momentum outputs | 1.152625 ms median `[1.025500, 1.276675]` | 1.3679 ms `[1.3523, 1.3840]` | 0.65495 ms (4 workers) | Rust serial 18.7% slower; Rust 4-worker 1.76× faster; stop tuning |
| Dry-air omega diagnosis | 2,686,976 omega outputs | 1.832250 ms median `[1.743500, 1.881850]` | 5.0201 ms `[4.9991, 5.0425]` | 0.66690 ms (16 workers) | Rust serial 2.74× slower; Rust 4-worker 1.38× faster; Rust 16-worker 2.75× faster; stop tuning |
| Moisture momentum coefficients | 7,819,264 coefficient outputs | 5.221150 ms median `[5.161350, 5.510350]` | 7.1239 ms `[7.0845, 7.1668]` | 2.0418 ms (4 workers) | Rust serial 36.4% slower; Rust 4-worker 2.56× faster; Rust 16-worker 1.52× faster; stop tuning |
| Full inverse density | 2,621,440 mass-point outputs | 0.210880 ms median `[0.206400, 0.223980]` | 0.32594 ms `[0.32076, 0.33097]` | 0.12102 ms (4 workers) | Rust serial 54.6% slower; Rust 4-worker 1.74× faster; stop tuning |
| Pressure-point geopotential | 2,621,440 mass-point outputs | 0.402140 ms median `[0.377740, 0.464480]` | 0.44482 ms `[0.44034, 0.44991]` | 0.14072 ms (4 workers) | Rust serial 10.6% slower; Rust 4-worker 2.86× faster; stop tuning |
| Integrated RK preparation | seven diagnostics on 2,621,440 mass points | 6.067100 ms median `[5.997000, 6.636100]` | 10.092 ms `[10.023, 10.162]` | 3.3025 ms (4 workers) | Rust serial 66.3% slower; Rust 4-worker 1.84× faster; Rust 16-worker 1.33× faster; stop tuning pending trajectory profile |
| Dry RK tendency assembly | 26,542,080 mutable values | 8.425600 ms median `[8.281500, 8.913450]` | 18.625 ms `[18.457, 18.845]` | 2.5235 ms (16 workers) | Rust serial 2.21× slower; Rust 4-worker 1.70× faster; Rust 16-worker 3.34× faster; stop tuning pending trajectory profile |
| Acoustic small-step preparation | 45,543,936 mutable values | 5.877800 ms median `[5.731300, 7.147600]` | 26.123 ms `[25.869, 26.388]` | 6.5595 ms (16 workers) | Rust serial 4.44× slower; Rust 4-worker 26.1% slower; Rust 16-worker 11.6% slower; operationally close, stop tuning pending trajectory profile |
| Kessler microphysics | 655,360 grid points | 31.7804 ms median `[31.2696, 33.4162]` | 30.944 ms `[30.601, 31.340]` | 5.0144 ms (16 workers) | Rust serial 2.6% faster; Rust 16-worker 6.34× faster; stop tuning |
| Classic NetCDF bulk write | 25 × 16 MiB field overwrites | 0.242086 s NetCDF-C | 0.543888 s | 0.543888 s | Rust 2.25× slower; Rust peak RSS 32% lower in separate run; gap recorded without bespoke serializer |

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
cargo bench -p wrf-dynamics --bench momentum_coupling -- --noplot
cargo bench -p wrf-dynamics --bench omega_diagnosis -- --noplot
cargo bench -p wrf-dynamics --bench moisture_coefficients -- --noplot
cargo bench -p wrf-dynamics --bench inverse_density -- --noplot
cargo bench -p wrf-dynamics --bench pressure_point_geopotential -- --noplot
cargo bench -p wrf-dynamics --bench runge_kutta_preparation -- --noplot
cargo bench -p wrf-dynamics --bench dry_tendency_assembly -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_step_preparation -- --noplot
cargo bench -p wrf-physics --bench kessler_microphysics -- --noplot
./scripts/benchmark-netcdf-restart.sh 1000
./scripts/benchmark-positive-definite-fortran.sh
./scripts/benchmark-held-suarez-fortran.sh
./scripts/benchmark-column-mass-staggering-fortran.sh
./scripts/benchmark-periodic-column-mass-fortran.sh
./scripts/benchmark-momentum-coupling-fortran.sh
./scripts/benchmark-omega-diagnosis-fortran.sh
./scripts/benchmark-moisture-coefficients-fortran.sh
./scripts/benchmark-inverse-density-fortran.sh
./scripts/benchmark-pressure-point-geopotential-fortran.sh
./scripts/benchmark-runge-kutta-preparation-fortran.sh
./scripts/benchmark-dry-tendency-assembly-fortran.sh
./scripts/benchmark-acoustic-step-preparation-fortran.sh
./scripts/benchmark-kessler-fortran.sh
```

The Rust detailed results live under `docs/performance/`. Fortran drivers live
beside their parity fixtures so workload mapping can be reviewed with the
scientific oracle.

## NetCDF restart I/O comparison notes

- WRF's storage backend is NetCDF-C, so the direct C comparison is more exact
  than adding a Fortran wrapper around the same calls.
- Both bulk writers create classic 64-bit-offset files and overwrite one
  256 × 256 × 64 `float` field 25 times from a reused caller allocation.
- A one-MiB `BufWriter` fixed the pure-Rust crate's per-value syscall behavior.
  The remaining scalar byte-order conversion gap is documented rather than
  hidden behind local unsafe code or an unproven SIMD serializer.
- See `docs/performance/netcdf-restart-2026-07-14.md` for memory, limitations,
  toolchains, and the tiny-schema control-plane run.

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
- The doubly periodic `calc_mu_uv` benchmark uses the same dimensions and
  storage, but its endpoints read periodic halos instead of physical copies.
  GNU Fortran 16.1.0 uses the same `-O3 -flto` tier as the earlier baseline.
- Periodic one-worker Rust is within 3.6% of serial Fortran and four-worker Rust
  is faster. Its warmed allocation profile is unchanged, so readability wins
  and no additional SIMD experiment is justified for this slice.

## Momentum-coupling comparison notes

- Both implementations process a 256 × 256 mass grid with 40 half levels and
  all three upper staggered boundaries, producing 7,950,336 outputs per call.
- Fortran uses eleven samples of 40 calls after 20 warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark. Inputs and outputs are reused.
- Replacing repeated global indexing with validated equal-length row slices
  preserved all 3,780 oracle bits and improved representative Rust timings by
  roughly 77%.
- One-worker Rust is in the same practical class as optimized Fortran and the
  default four-worker path is faster. Five 1,520-byte scheduler allocations per
  100 calls are independent of field size; no numerical scratch is allocated.
- No explicit SIMD or target-specific tuning is justified without an
  end-to-end profile identifying this routine as a material hotspot.

## Omega-diagnosis comparison notes

- Both implementations diagnose 2,686,976 complete-column outputs on a
  256 × 256 × 40 mass grid with identical halos, coefficients, map factors,
  and grid spacing.
- Fortran uses eleven samples of 20 calls after ten warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark. All fields are reused.
- The first parity-correct column-strided Rust version measured 17.960 ms with
  one worker. Validated equal-length west-east row views preserved all 1,960
  oracle values and improved accepted serial time by about 72%.
- One-worker Rust remains slower than serial Fortran, but the standard
  multithreaded path is faster at both four and 16 workers. Settled execution
  uses one 1,520-byte scheduler allocation per 100 calls and no numerical
  scratch.
- No explicit SIMD is justified until integrated profiling identifies this
  routine as a material limiter.

## Moisture-coefficient comparison notes

- Both implementations process a 256 × 256 × 40 mass grid, six active
  moisture species, and all three upper stagger points, producing 7,819,264
  coefficients per call.
- WRF's generated scalar padding slot is present and poisoned in Fortran but is
  omitted from the Rust active-species view. Both sides accumulate the same six
  physical fields in the same order.
- Fortran uses eleven samples of 20 calls after ten warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark. Inputs and outputs are reused.
- Rust uses each output row as the temporary species total, replacing WRF's
  automatic `qtot` row without numerical scratch or reassociation.
- Four-worker Rust is the fastest measured configuration. The all-16-worker
  result is slower than four workers but remains 1.52× faster than
  serial Fortran. Five 1,520-byte scheduler allocations occur per 100 calls at
  every worker count, with no reallocations or numerical scratch.
- The standard multithreaded path is competitive, so explicit SIMD and custom
  scheduling are not justified without an integrated ARW profile.

## Full inverse-density comparison notes

- Both implementations add perturbation and base-state inverse density at
  2,621,440 active points on a 256 × 256 × 40 mass grid. Inputs, output, halos,
  and all three upper stagger points are allocated once and reused.
- Fortran uses eleven samples of 50 calls after 20 warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark.
- The Rust hot loop is a safe contiguous slice addition. This keeps the source
  readable and exposes ordinary compiler autovectorization without an explicit
  SIMD implementation.
- Four-worker Rust is 1.74× faster than serial Fortran. Sixteen workers lose to
  four because this small three-stream kernel is memory-bandwidth and dispatch
  limited.
- Settled execution records one 1,520-byte scheduler allocation per 100 calls,
  no reallocations, and no numerical scratch. The standard multithreaded path
  is competitive, so additional SIMD and scheduling work stops here.

## Pressure-point geopotential comparison notes

- Both implementations average base-state and perturbation geopotential from
  adjacent full levels into 2,621,440 active pressure points on a
  256 × 256 × 40 mass grid.
- Fortran uses eleven samples of 50 calls after 20 warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark. All three fields are reused.
- Rust preserves WRF's base-state-first four-term single-precision addition
  order. The exact oracle includes a reassociation-sensitive overflow case.
- One-worker Rust is 10.6% slower than optimized serial Fortran. Four-worker
  Rust is 2.86× faster; 16 workers add overhead and are 1.6% slower than
  Fortran.
- Settled execution records one 1,520-byte scheduler allocation per 100 calls,
  no reallocations, and no numerical scratch. The ordinary multithreaded path
  clears the performance gate, so explicit SIMD work stops here.

## Integrated Runge-Kutta preparation comparison notes

- Both implementations run the exact seven-stage `rk_step_prep` diagnostic
  sequence on a 256 × 256 × 40 mass grid with two active moisture species,
  upper stagger storage, and reused inputs and outputs.
- GNU Fortran 16.1.0 uses `-O3 -flto`, eleven samples of 20 calls, and ten
  warm-up calls. Rust uses the workspace bench profile (`opt-level=3`, ThinLTO,
  one codegen unit) and Criterion. Neither side enables fast-math or a native
  CPU flag.
- One-worker Rust is 66.3% slower than serial Fortran. Four-worker Rust is
  1.84× faster, while the standard 16-worker host path is 1.33× faster. This is
  accepted without cross-stage fusion or custom SIMD because normal parallel
  execution clears the gate and no coupled trajectory profile identifies a
  model-level bottleneck.
- Every 100 settled calls records 19 scheduler allocations totaling 28,880
  bytes, no reallocations, no numerical scratch, and no full-field clones.
  Preflight validation only borrows existing descriptors and fields.

## Dry Runge-Kutta tendency assembly comparison notes

- Both implementations execute first-substep `rk_addtend_dry` on a 256 × 256 ×
  40 mass grid, including the upper W/geopotential level and reused fields.
- GNU Fortran 16.1.0 uses `-O3 -flto`; Rust uses portable optimization level 3,
  ThinLTO, and one codegen unit. Neither enables fast-math or a native CPU flag.
- One-worker Rust is 2.21× slower than serial Fortran. Four workers are 1.70×
  faster, and the standard 16-worker host path is 3.34× faster.
- The safe paired-output scheduler keeps each RK/persistent pair in one memory
  pass. Every 100 settled calls records nine allocations totaling 13,680 bytes,
  no reallocations, no numerical scratch, and no field clones.
- FatLTO produced no statistically detectable improvement over ThinLTO in the
  same Criterion run. Parallel Rust already clears the gate, so explicit SIMD,
  target-specific flags, and a more complex fused scheduler stop here.

## Acoustic small-step preparation comparison notes

- Both implementations execute first-substep `small_step_prep` on a 256 × 256
  × 40 mass grid with upper U/V staggers and the full upper W/geopotential
  level, writing 45,543,936 values per call.
- GNU Fortran 16.1.0 uses `-O3 -flto`; Rust uses portable optimization level 3,
  ThinLTO, and one codegen unit. Neither enables fast-math or a native CPU flag.
- One-worker Rust is 4.44× slower than serial Fortran. Four-worker Rust is
  26.1% slower and the default 16-worker path is 11.6% slower.
- Every 100 settled calls records 28 scheduler allocations totaling 42,560
  bytes, no reallocations, no numerical scratch, and no field clones.
- Ordinary parallel Rust is operationally close to optimized Fortran. Per the
  project stopping rule, explicit SIMD and more complex pass fusion wait for a
  coupled trajectory profile.

## Kessler microphysics comparison notes

- Date, machine, and toolchains match the other 2026-07-13 comparisons.
- Both implementations process 128 × 128 × 40 points with the same mixed
  vapor/cloud/rain initialization and 60-second physics time step.
- Mutable state restoration occurs before every individually timed call and is
  excluded from both measurements.
- Fortran uses eleven samples of five calls after three warm-up calls. Rust uses
  Criterion's 100-sample statistical benchmark.
- One-worker Rust and serial Fortran are within normal operational proximity.
  No SIMD screen is justified without a model-level profile.
- The reusable Rust workspace holds about 2.62 MB for this domain. Settled
  execution records three 1,520-byte scheduler allocations per 100 calls and
  no numerical scratch allocation or reallocation.
