# Rust and Fortran performance tracking

> **Historical snapshot.** The comparison policy and decision records below
> remain useful, but the current suite catalog and measured values are no
> longer hand-maintained here. See the generated
> [benchmark catalog](docs/generated/benchmark-catalog.md), post-merge Actions
> receipts, and the [latest GitHub Pages dashboard](https://mikeortman.github.io/wrf-rs/).

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
| Acoustic pressure, nonhydrostatic | 2,621,440 mass points | 1.529500 ms median `[1.512550, 2.006750]` | 1.8319 ms `[1.8075, 1.8596]` | 0.81126 ms (4 workers) | Rust serial 19.8% slower; Rust 4-worker 1.89× faster; stop tuning pending trajectory profile |
| Acoustic pressure, hydrostatic | 2,621,440 mass points | 1.602750 ms median `[1.563400, 1.765500]` | 2.0816 ms `[2.0569, 2.1137]` | 0.95950 ms (4 workers) | Rust serial 29.9% slower; Rust 4-worker 1.67× faster; stop tuning pending trajectory profile |
| Vertical acoustic coefficients | 256 × 256 × 40 columns | 1.867500 ms median `[1.829250, 1.956000]` | 14.608 ms `[14.536, 14.686]` | 1.7109 ms (16 workers) | Rust serial 7.82× slower; Rust 16-worker 1.09× faster; stop tuning pending trajectory profile |
| Acoustic horizontal momentum | 256 × 256 × 40 mass grid plus U/V staggers | 7.569 ms median | 71.852 ms | 7.802 ms (16 workers) | Rust 16-worker 3.1% slower; operationally close, stop tuning pending trajectory profile |
| Acoustic mass, omega, and theta | 256 × 256 × 40 mass grid | 5.368 ms median | 29.960 ms | 4.241 ms (16 workers) | Rust 16-worker 1.27× faster; stop tuning pending trajectory profile |
| Implicit acoustic vertical momentum | 256 × 256 × 40 mass grid plus top level | 16.745 ms median `[16.014, 26.746]` | 61.295 ms `[61.074, 61.480]` | 6.621 ms (16 workers) | Rust 4-worker 1.04× faster; Rust 16-worker 2.53× faster; stop tuning pending trajectory profile |
| Acoustic flux accumulation | three substeps on 256 × 256 × 40 mass grid plus U/V/W staggers | 5.513750 ms median `[5.163000, 6.643500]` | 26.048 ms `[25.922, 26.187]` | 3.6192 ms (16 workers) | Rust 16-worker 1.52× faster; stop tuning pending trajectory profile |
| Specified-boundary tendency update | 200,800 updated points on 256 × 256 × 40 mass grid | 0.144940 ms median `[0.133350, 0.162710]` | 0.096968 ms `[0.096231, 0.097789]` | 0.037367 ms (4 workers) | Rust serial 1.49× faster; Rust 4-worker 3.88× faster; default 16-worker 1.61× faster; stop tuning |
| Specified-boundary geopotential update | 205,820 updated points on 256 × 256 × 41 full-level grid | 0.310400 ms median `[0.301490, 0.337520]` | 0.15437 ms `[0.15322, 0.15570]` | 0.055178 ms (4 workers) | Rust serial 2.01× faster; Rust 4-worker 5.63× faster; default 16-worker 3.25× faster; stop tuning |
| Zero-gradient specified boundary | 205,820 copied points on 256 × 256 × 41 full-level grid | 0.134000 ms median `[0.131830, 0.148520]` | 0.14306 ms `[0.14221, 0.14406]` | 0.11030 ms (4 workers) | Rust serial 6.8% slower; Rust 4-worker 1.21× faster; close enough, stop tuning |
| Flow-dependent specified boundary | 200,800 classified points on 256 × 256 × 40 mass grid | 0.189720 ms median `[0.188190, 0.305730]` | 0.18688 ms `[0.18611, 0.18778]` | 0.15356 ms (4 workers) | Rust serial 1.5% faster; Rust 4-worker 1.24× faster; stop tuning |
| Flow-dependent constant inflow | 200,800 classified points on 256 × 256 × 40 mass grid | 0.187580 ms median `[0.186440, 0.256330]` | 0.21569 ms `[0.21443, 0.21714]` | 0.17217 ms (4 workers) | Rust serial 15.0% slower; Rust 4-worker 1.09× faster; close enough, stop tuning |
| Flow-dependent preserved inflow | 200,800 classified points on 256 × 256 × 40 mass grid | 0.177190 ms median `[0.170250, 0.186570]` | 0.21686 ms `[0.21578, 0.21814]` | 0.17267 ms (4 workers) | Rust serial 22.4% slower; Rust 4-worker 2.6% faster; close enough, stop tuning |
| Specified-boundary finalization | 205,820 reconstructed points on 256 × 256 × 41 full-level grid | 0.157440 ms median `[0.153350, 0.287200]` | 0.30229 ms `[0.30036, 0.30439]` | 0.10483 ms (4 workers) | Rust serial 92.0% slower; Rust 4-worker 1.50× faster; default 16-worker 1.36× faster; stop tuning |
| Specified-boundary tendency assignment | 200,800 copied points on 256 × 256 × 40 mass grid | 0.082900 ms median `[0.079250, 0.089420]` | 0.069473 ms `[0.069133, 0.069811]` | 0.028529 ms (4 workers) | Rust serial 1.19× faster; Rust 4-worker 2.91× faster; default 16-worker within 3.2%; stop tuning |
| Specified-boundary relaxation | 238,080 five-point updates on 256 × 256 × 40 mass grid | 0.407650 ms median `[0.399020, 0.471110]` | 1.355342 ms | 0.465916 ms (16 workers) | Rust serial 3.33× slower; Rust 4-worker within 15.0%; default 16-worker within 14.3%; operationally close, stop tuning |
| Complete dry boundary relaxation | 1,209,216 five-point updates plus 7,995,392 mass-weighted points on 256 × 256 × 40 mass grid | 4.1218 ms median `[3.9272, 4.4852]` | 20.101 ms | 4.1620 ms (16 workers) | Default 16-worker Rust within 1.0%; reusable workspace and no field clones; close enough, stop tuning |
| Complete dry boundary-tendency assignment | 1,019,860 copied points across U/V/PH/T/MU/W on 256 × 256 × 40 mass grid | 0.485370 ms median `[0.448310, 0.504650]` | 0.48471 ms | 0.18404 ms (4 workers) | Serial effectively tied; Rust 4-worker 2.64× faster; default 16-worker within 3.3%; stop tuning |
| Coupled dry tendency and boundary stage | `rk_addtend_dry` then nested `spec_bdy_dry` on 256 × 256 × 40 mass grid | 9.029150 ms median `[8.894750, 9.238400]` | 19.995 ms `[19.927, 20.068]` | 3.9960 ms (16 workers) | Rust 4-worker 1.55× faster; Rust 16-worker 2.26× faster; no extra SIMD or fusion |
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
cargo bench -p wrf-dynamics --bench dry_tendency_boundary_stage -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_step_preparation -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_pressure -- --noplot
cargo bench -p wrf-dynamics --bench vertical_acoustic_coefficients -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_horizontal_momentum -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_mass_theta -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_vertical_momentum -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_flux_accumulation -- --noplot
cargo bench -p wrf-dynamics --bench zero_gradient_boundary -- --noplot
cargo bench -p wrf-dynamics --bench flow_dependent_boundary -- --noplot
cargo bench -p wrf-dynamics --bench flow_dependent_inflow_policies -- --noplot
cargo bench -p wrf-dynamics --bench specified_boundary_finalization -- --noplot
cargo bench -p wrf-dynamics --bench specified_boundary_tendencies -- --noplot
cargo bench -p wrf-dynamics --bench specified_boundary_relaxation -- --noplot
cargo bench -p wrf-dynamics --bench dry_boundary_relaxation -- --noplot
cargo bench -p wrf-dynamics --bench dry_boundary_tendencies -- --noplot
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
./scripts/benchmark-dry-tendency-boundary-stage-fortran.sh
./scripts/benchmark-acoustic-step-preparation-fortran.sh
./scripts/benchmark-acoustic-pressure-fortran.sh
./scripts/benchmark-vertical-acoustic-coefficients-fortran.sh
./scripts/benchmark-acoustic-horizontal-momentum-fortran.sh
./scripts/benchmark-acoustic-mass-theta-fortran.sh
./scripts/benchmark-acoustic-vertical-momentum-fortran.sh
./scripts/benchmark-acoustic-flux-accumulation-fortran.sh
./scripts/benchmark-zero-gradient-boundary-fortran.sh
./scripts/benchmark-flow-dependent-boundary-fortran.sh
./scripts/benchmark-flow-dependent-inflow-policies-fortran.sh
./scripts/benchmark-specified-boundary-finalization-fortran.sh
./scripts/benchmark-specified-boundary-tendencies-fortran.sh
./scripts/benchmark-specified-boundary-relaxation-fortran.sh
./scripts/benchmark-dry-boundary-relaxation-fortran.sh
./scripts/benchmark-dry-boundary-tendencies-fortran.sh
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

## Acoustic pressure diagnosis comparison notes

- Both implementations process 256 × 256 × 40 mass points and retain the upper
  geopotential level. Nonhydrostatic and hydrostatic modes use the same reused
  fields and pressure-history initialization phase.
- GNU Fortran 16.1.0 uses `-O3 -flto`; Rust uses portable optimization level 3,
  ThinLTO, and one codegen unit. Neither enables fast-math or a native CPU flag.
- Four-worker Rust is 1.89× faster in nonhydrostatic mode and 1.67× faster in
  hydrostatic mode than optimized serial Fortran.
- Reordering the parity-correct hydrostatic recurrence from column-strided to
  WRF-like level-major traversal improved serial Rust from 8.63 to 2.08 ms
  without changing any oracle bit.
- Settled 100-call phases record at most five allocations and 6,080 bytes, no
  reallocations, no numerical scratch, and no field clones.
- The normal multithreaded path clears the gate, so explicit SIMD, unsafe
  fusion, and custom per-kernel worker selection wait for a coupled trajectory
  profile.

## Vertical acoustic coefficient comparison notes

- Both implementations construct `a`, `alpha`, and `gamma` for 256 × 256
  columns with 40 mass half levels and the additional top full level.
- GNU Fortran 16.1.0 uses `-O3 -flto`; Rust uses portable optimization level 3,
  ThinLTO, and one codegen unit. Neither enables fast-math or a native CPU flag.
- One-worker Rust is 7.82× slower and four-worker Rust is 2.08× slower than
  serial Fortran. The standard 16-worker host path is 1.09× faster.
- Reordering the first parity-correct column-strided traversal to WRF-like
  level-major contiguous-X traversal preserved all 3,024 oracle values and
  improved serial Rust from 27.881 to 14.608 ms.
- Every 100 settled calls records three scheduler allocations totaling 4,560
  bytes, no reallocations, no numerical scratch, and no field clones.
- The default path clears the gate. The remaining serial vectorization gap is
  recorded for integrated profiling rather than addressed with speculative
  SIMD during this port slice.

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

## Acoustic horizontal-momentum comparison notes

- Both implementations run nonhydrostatic `advance_uv` over a 256 × 256 × 40
  mass grid with upper U/V stagger points and guard storage.
- GNU Fortran 16.1.0 uses `-O3 -flto`; Rust uses optimization level 3,
  ThinLTO, and one codegen unit. Neither enables fast-math or native-CPU flags.
- Serial Fortran measured 7.569 ms median. Rust measured 71.852 ms with one
  worker, 18.850 ms with four, and 7.802 ms with 16 workers.
- The normal host-parallel path is 3.1% slower than optimized serial Fortran,
  which is operationally close. Explicit SIMD and more complex fusion stop
  here until a coupled profile identifies a material bottleneck.
- Rust removes the tile-sized `dpn`, `dpxy`, and `mudf_xy` numerical scratch by
  evaluating interpolation, pressure-gradient, and damping terms directly.

## Acoustic mass, omega, and theta comparison notes

- Both implementations run `advance_mu_t` over a 256 × 256 × 40 mass grid with
  upper U/V/full-level storage and the same constant-valued numerical fixture.
- GNU Fortran 16.1.0 uses `-O3 -flto`; Rust uses optimization level 3,
  ThinLTO, and one codegen unit. Neither enables fast-math or native-CPU flags.
- Serial Fortran measured 5.368 ms median. Rust measured 29.960 ms with one
  worker, 8.070 ms with four, and 4.241 ms with 16 workers.
- The standard host-parallel path is 1.27× faster than optimized serial
  Fortran. SIMD and more elaborate multi-output scheduling stop here.
- Rust reuses required diagnostic outputs as short-lived divergence and prior-
  mass scratch, then writes their specified final values. It allocates no
  numerical scratch and clones no fields.

## Implicit acoustic vertical-momentum comparison notes

- Both implementations run `advance_w` over a 256 × 256 × 40 mass grid with
  the full upper vertical-momentum level, gradient-first geopotential
  advection, nonrigid top, terrain lower boundary, tridiagonal sweeps, and
  upper damping.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Serial Fortran measured 16.745 ms median. Rust measured 61.295 ms with one
  worker, 16.084 ms with four, and 6.621 ms with 16 workers.
- Four-worker Rust is 4.1% faster than serial Fortran, and the standard
  16-worker path is 2.53× faster. SIMD and column-layout changes stop here.
- The guarded workload's reusable RHS is 10.67 MiB. Every 100 settled calls
  records four scheduler allocations totaling 6,080 bytes, four matching
  deallocations, no reallocations, no field allocation, and no clones.

## Complete local acoustic-trajectory estimate

- The exact trajectory call count is one preparation, four nonhydrostatic
  pressure diagnoses, one coefficient construction, three horizontal, mass,
  and vertical advances, and one three-call flux sequence.
- Summing the matched 256 × 256 × 40 stage medians gives 108.423 ms for
  optimized serial Fortran, 563.428 ms for one-worker Rust, 150.770 ms for
  four-worker Rust, and 73.949 ms for 16-worker Rust.
- The standard host-parallel estimate is 1.47× faster than serial Fortran.
- This is an arithmetic composition of measured stage medians, not a new fused
  wall-clock benchmark. Both sides exclude communication and boundary work.
  Optimization levels remain equivalent: Fortran `-O3 -flto`, Rust level 3
  with ThinLTO and one codegen unit, no fast-math or native CPU flag.
- The ordinary path clears the performance gate, so SIMD and fusion stop. A
  direct integrated measurement waits for the communication/boundary driver.

## Specified-boundary tendency comparison notes

- Both implementations update the 200,800 trapezoid and side-zone points on a
  256 × 256 × 40 mass grid with a five-point specified zone.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Serial Rust measures 0.096968 ms versus Fortran's 0.144940 ms median. Four
  workers measure 0.037367 ms; the default 16-worker path measures 0.089821 ms.
- The first parity-correct Rust version scanned complete horizontal planes and
  measured 8.2470 ms serially. Direct side ranges preserve every oracle bit and
  remove work outside the specified zones.
- Every 100 settled calls records at most four scheduler allocations totaling
  3,152 bytes, no reallocations, no numerical scratch, and no field clones.
- The ordinary scalar/parallel path is already faster than optimized serial
  Fortran, so explicit SIMD and custom worker selection stop here.

## Specified-boundary geopotential comparison notes

- Both implementations update 205,820 full-level points in a five-point zone
  on the same 256 × 256 horizontal mass domain.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Serial Rust measures 0.15437 ms versus Fortran's 0.310400 ms median. Four
  workers measure 0.055178 ms; the default 16-worker path measures 0.095639 ms.
- Rust computes the prior column mass as a scalar at the consuming point. It
  does not declare WRF's tile-sized `mu_old` automatic array, allocate numerical
  scratch, or clone fields.
- Every 100 settled calls records two scheduler allocations totaling 3,040
  bytes and no reallocations.
- The normal scalar/parallel implementation clears the performance gate, so
  explicit SIMD and custom worker selection stop here.

## Zero-gradient specified-boundary comparison notes

- Both implementations copy 205,820 full-level points from the nearest
  independent interior row or column on a 256 × 256 domain with a five-point
  specified zone.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Serial Rust measures 0.14306 ms versus Fortran's 0.134000 ms median. Four
  workers measure 0.11030 ms; the default 16-worker path measures 0.25671 ms.
- Every 100 settled calls records three scheduler allocations totaling 4,560
  bytes, no reallocations, no numerical scratch, and no field clones.
- Serial Rust is within ordinary practical parity and four-worker Rust is
  faster. The host-default pool is overhead-bound for this thin perimeter
  kernel, but per-kernel worker selection would complicate the backend for no
  demonstrated end-to-end benefit. SIMD and custom scheduling stop here.

## Flow-dependent specified-boundary comparison notes

- Both implementations classify and write 200,800 mass-level boundary points
  from coupled U/V signs on a 256 × 256 × 40 domain with a five-point zone.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Serial Rust measures 0.18688 ms versus Fortran's 0.189720 ms median. Four
  workers measure 0.15356 ms; the default 16-worker path measures 0.32330 ms.
- Every 100 settled calls records three scheduler allocations totaling 4,560
  bytes, no reallocations, no numerical scratch, and no field clones.
- Serial and four-worker Rust clear the practical performance gate. The
  host-default pool is overhead-bound for the thin perimeter, but custom
  scheduling is deferred until an integrated moisture/TKE/scalar profile shows
  material value. Explicit SIMD also stops here.

## Flow-dependent inflow-policy comparison notes

- `flow_dep_bdy_qnn` and `flow_dep_bdy_fixed_inflow` use the same 200,800-point
  workload and mixed U/V signs as the base flow-dependent comparison.
- Constant-inflow Fortran measures 0.187580 ms median. Rust measures 0.21569 ms
  with one worker, 0.17217 ms with four, and 0.31029 ms with 16.
- Preserve-inflow Fortran measures 0.177190 ms median. Rust measures 0.21686 ms
  with one worker, 0.17267 ms with four, and 0.30735 ms with 16.
- Each policy records three scheduler allocations and 4,560 bytes across 100
  settled calls, with no reallocations, numerical scratch, or field clones.
- The Rust capability pays one explicit policy branch instead of maintaining
  three copied loop families. Four-worker Rust is competitive or faster for
  both policies, so the readability and maintenance benefit wins without
policy specialization, custom scheduling, or explicit SIMD.

## Specified-boundary finalization comparison notes

- Both implementations reconstruct 205,820 vertical-momentum points on a
  256 × 256 × 41 full-level domain with a five-point specified zone and eight
  stored boundary points.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Fortran measures 0.157440 ms median. Rust measures 0.30229 ms with one
  worker, 0.10483 ms with four, and 0.11565 ms with the default 16.
- Every 100 settled calls records one scheduler allocation totaling 1,520
  bytes, no reallocations, no numerical scratch, and no field clones.
- Four-worker and host-default Rust both exceed the optimized serial source.
  The serial gap is recorded, but specialization, custom scheduling, and
  explicit SIMD stop unless integrated profiling identifies material value.

## Specified-boundary tendency-assignment comparison notes

- Both implementations assign 200,800 boundary-file tendencies on a
  256 × 256 × 40 mass grid with a five-point specified zone and eight stored
  boundary points.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Fortran measures 0.082900 ms median. Rust measures 0.069473 ms with one
  worker, 0.028529 ms with four, and 0.085521 ms with the default 16.
- Every 100 settled calls records one scheduler allocation totaling 1,520
  bytes, no reallocations, no numerical scratch, and no field clones.
- Serial and four-worker Rust exceed the optimized serial source. The default
  host pool is only 3.2% slower on this thin perimeter copy, so custom worker
  selection and explicit SIMD stop pending an integrated boundary-driver
  profile.

## Specified-boundary relaxation comparison notes

- Both implementations apply 238,080 five-point updates on a 256 × 256 × 40
  mass grid with one fixed specified point, six relaxed points, and eight
  stored boundary points.
- GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`; Rust uses optimization
  level 3, ThinLTO, and one codegen unit. Neither enables fast-math or native-
  CPU flags.
- Fortran measures 0.407650 ms median. Rust measures 1.355342 ms with one
  worker, 0.468761 ms with four, and 0.465916 ms with the default 16.
- Hoisting side selection and boundary slice lookup out of the point loop cut
  serial Rust from 3.193886 ms without changing one oracle bit.
- Every 100 settled calls records one scheduler allocation totaling 1,520
  bytes, no reallocations, no numerical scratch, and no field clones.
- Four-worker and host-default Rust are within 15% of optimized serial Fortran.
  That is operationally close for the default multithreaded path, so explicit
  SIMD and more duplicated side specialization stop pending an integrated
  boundary-driver profile.
