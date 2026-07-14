# Current implementation state

Last updated: 2026-07-13

This file is the durable handoff for continuing the WRF Rust port after context
compaction or a new session. Update it only with verified current-state facts.

## Goal and non-negotiable constraints

- Reimplement WRF in Rust and prove parity by porting and differentially running
  upstream tests.
- Follow `RUST_BACKEND_STYLE_GUIDE.md` as the governing Rust contract.
- Implement CPU kernels first.
- Multithreading is the standard runtime path, not an opt-in feature.
- Keep storage and kernel capability boundaries compatible with a future native
  GPU backend.
- Optimize release-mode compute and memory behavior without sacrificing clear
  ownership, names, and structure. Cheap descriptor/`Arc` clones are acceptable;
  field and domain-state clones in hot paths are not.
- Treat SIMD as a per-kernel CPU implementation layer after scalar parity.
- Group source by scientific family with nested modules; keep crate roots as
  stable public facades rather than flat implementation indexes.
- Benchmark the exact pinned Fortran routine on a matched workload for every
  numerical kernel and track honest Rust/Fortran ratios.

## Upstream reference

- Repository: `https://github.com/wrf-model/WRF`
- Tag: `v4.7.1`
- Commit: `f52c197ed39d12e087d02c50f412d90d418f6186`
- Archive SHA-256:
  `7227916c7871cec36a0a1bf23619fe6d29664474679c8207b4c6f22b10cbab6b`
- Local source: `upstream/WRF` (ignored by the root Git repository)
- Reproducer: `scripts/fetch-wrf.sh`

The GitHub release archive omits WRF's nested submodule contents. Their exact
commits and archive checksums are recorded in `UPSTREAM.toml`, and
`scripts/fetch-wrf.sh` fetches them into `phys/noahmp`, `phys/MYNN-EDMF`, and
`.ci/hpc-workflows`.

## Implemented crates

### `wrf-time`

Implemented:

- proleptic Gregorian dates including year zero and negative years;
- exact rational-second model timestamps and fixed intervals;
- normalized WRF-compatible formatting;
- time/interval addition, subtraction, scaling, and truncating ratios;
- deterministic fixed-step clocks;
- typed errors and component types.

Tests are at the bottom of their implementation files per the style guide. A
previous false expectation for negative fractional interval formatting was
corrected to WRF's `-NN/DD` convention.

### `wrf-compute`

Implemented:

- checked three-dimensional grid shapes;
- contiguous copyable-scalar CPU fields;
- backend-owned field storage boundary;
- narrow `ComputeBackend` allocation trait;
- persistent Rayon CPU thread pool using all available host parallelism by
  default;
- disjoint mutable output chunks with immutable shared inputs;
- typed kernel and worker failures;
- cheap backend clones sharing one `Arc<ThreadPool>`.
- exact contiguous-block scheduling for indivisible grid lines and profiles;
- typed validation of block shapes and worker-panic containment.

Future numerical crates must define narrow kernel capability traits. Do not put
unrelated kernels into `ComputeBackend` and do not expose arbitrary CPU closures
as the GPU API.

### `wrf-dynamics`

Implemented:

- the focused `PositiveDefiniteKernels` backend capability;
- WRF `positive_definite_sheet` for single-precision CPU fields;
- WRF `positive_definite_slab` with typed, validated active ranges;
- in-place correction with no temporary line allocation or copies;
- independent west-east lines scheduled through the persistent Rayon pool;
- scalar ordered minimum/sum reductions for exact parity;
- typed sheet-shape, total-count, and worker failures.
- the focused `HeldSuarezDampingKernels` backend capability;
- WRF `held_suarez_damp` over six borrowed fields with no field clones;
- typed validation of shapes, ranges, the pressure reference level, and both
  C-grid staggered predecessors;
- exact-bit tendency updates over independently scheduled west-east lines;
- scientific-family directories for `positive_definite` and `held_suarez`,
  with `lib.rs` retained as the stable public facade.
- all interior, lower-boundary, upper-boundary, and both-boundaries paths of
  ARW `calc_mu_staggered`, exposed through `ColumnMassStaggeringKernels`;
- typed separation of allocated memory shape, physical mass-domain ranges, and
  active momentum-tile ranges, with boundary contact and cross-axis clipping
  derived rather than supplied as boolean flags;
- parallel, disjoint west-east-major output rows with immutable shared mass
  inputs and no field-sized scratch;
- exact-body Fortran extraction oracle with 240 raw-bit output/sentinel checks
  across all eight axis/path combinations, plus validation-before-mutation and
  one/four-worker determinism tests at all physical boundaries.

The differential drivers compile the pinned upstream Fortran module directly.
The sheet covers nine exact-bit cases, including signed zero and the epsilon
branch. The slab fixture covers non-one memory origins, domain/tile clipping,
correction branches, and unchanged halo/stagger sentinels. Rust also proves
sheet bitwise determinism between one and four workers.

The Held-Suarez differential fixture checks 16 active and boundary values with
non-one memory origins. Added Rust tests cover one/four-worker determinism,
shape failure before mutation, all range categories, staggered predecessors,
and the pressure reference level.

The non-periodic column-mass staggering routine-level paths are complete for the
current deterministic corpus. Interior subdomain tiles use halo mass points;
physical lower and upper boundaries copy the nearest full mass exactly as WRF
does. Its matched benchmark and allocation budget are complete. The next gates
are randomized differential inputs, exceptional-float policy, periodic
`calc_mu_uv` variants, and idealized-case integration.

## Performance decisions

- Release profile: thin LTO, one codegen unit.
- Allocate fields and scratch buffers during setup, not timesteps.
- Prefer contiguous structure-of-arrays field layouts.
- Preserve WRF precision and operation order until parity is established.
- `pulp` 0.22.3 is the accepted stable-Rust SIMD layer for Held-Suarez damping
  because it supports one runtime ISA dispatch per kernel. It remains subject
  to per-kernel evidence; `wide` is still a controlled-target candidate.
- `std::simd` is not the stable production baseline while it requires nightly.
- SIMD dispatch happens once per kernel and runs inside CPU worker chunks.
- Criterion 0.7 is a dev-only statistical benchmark dependency; 0.8 is excluded
  because it exceeds the workspace's declared Rust 1.85 minimum.

See `docs/architecture/compute_backends.md`,
`docs/architecture/performance_principles.md`, and
`docs/architecture/simd.md`.

## Positive-definite performance baseline

The 1,048,576-value Criterion baseline on an Apple M3 Max measured:

- sheet: 1.1569 ms (1 worker), 316.00 µs (4), 274.85 µs (16);
- slab: 1.8084 ms (1 worker), 483.08 µs (4), 347.98 µs (16).

Four workers capture most sheet scaling; 16 workers reach 4.21× sheet and 5.20×
slab speedup over one. AArch64 release assembly contains scalar rather than
packed NEON arithmetic in the pointwise loops. See
`docs/performance/positive-definite-2026-07-13.md` for environment, confidence
intervals, throughput, and caveats. End-to-end steady-state allocation
measurement shows one or two 1,520-byte Rayon/crossbeam queue allocations per
100 dispatches, independent of worker count, and no reallocations. Do not claim
zero Rayon-dispatch allocations; the precise measured amortized cost is
0.01–0.02 allocations and 15.2–30.4 bytes per million-point call.

`pulp` 0.22.3 was then prototyped for only the pointwise passes. It preserved
exact bits across 14 vector/tail lengths and the upstream fixtures, but slowed
the one- and four-worker benchmarks by about 1–4%. Gains appeared only at 16
workers. The implementation and dependency were removed; keep the scalar path
for this kernel. `pulp` remains a candidate for more pointwise-dominant kernels.

Matched GNU Fortran 14.2.0 `-O3 -flto` medians are 1.709219 ms for
`positive_definite_sheet` and 2.336656 ms for `positive_definite_slab` on the
same 1,048,576-value all-correction workload. One-worker Rust is respectively
1.48× and 1.29× faster; 16-worker Rust is 6.22× and 6.71× faster than serial
Fortran. These are combined routine results, not isolated attribution of scratch
copies versus the repeated negativity scan.

## Held-Suarez performance baseline

For 2,097,152 momentum updates on the Apple M3 Max, accepted safe SIMD measured
0.93459 ms with one Rust worker, 0.29105 ms with four, and 0.52122 ms with 16. A
matched GNU Fortran 14.2.0 `-O3 -flto` run of the pinned routine measured a
0.859712 ms median across seven samples. Current Rust is 8.7% slower serially and 2.95×
faster with four workers than serial Fortran. This is an isolated-kernel result,
not a whole-model claim. See `PERFORMANCE_PARITY.md` and
`docs/performance/held-suarez-2026-07-13.md`.

The `pulp` implementation preserves exact scalar bits for every tested line
length from 1 through 257 and improves the scalar baseline by 4.4–5.4% across
worker counts. Its warmed two-pass dispatch uses three 1,520-byte scheduler
allocations per 100 calls, with no reallocations or numerical scratch.

Bench-only native CPU and/or fat-LTO builds were screened. Native+fat gained
1.9% on serial Held-Suarez but regressed its four- and 16-worker cases. It was
flat to slightly slower for positive-definite at one and four workers, with
gains limited to the noisier 16-worker cases. The separated settings also failed
to improve representative worker counts consistently. Keep the portable
ThinLTO production/benchmark baseline.

## Column-mass staggering performance baseline

For 2,099,200 momentum-mass outputs on a 1,024 × 1,024 physical domain, scalar
Rust measured 332.80 µs with one worker, 115.32 µs with four, and 242.03 µs with
16. Matched GNU Fortran 14.2.0 `-O3 -flto` measured a 286.850 µs median across
eleven samples. Rust is 16.0% slower serially and 2.49× faster with four workers
than serial Fortran. These are isolated-routine results.

Each warmed 100-dispatch phase at 1, 4, and 16 workers recorded three 1,520-byte
scheduler allocations, or 4,560 bytes total, with no reallocations and no
numerical scratch. A parity-correct `pulp` prototype regressed representative
one- and four-worker results and was removed. A safe slice-iterator formulation
also regressed serial performance and was removed. Keep the readable scalar
implementation. See `docs/performance/column-mass-staggering-2026-07-13.md`.

## WRF time oracle

The bundled Fortran `external/esmf_time_f90/Test1.F90` is compiled locally
with Homebrew `gfortran` and its output matched `Test1.out.correct` exactly.

Two build details are required:

1. preprocess with `TIME_F90_ONLY` and compile generated `.f` files as free
   form;
2. replace `defaultCalendar=` with the implementation's actual
   `defaultcalkind=` keyword in the generated test copy only.

The second item is an upstream v4.7.1 test/interface mismatch. Never patch the
pinned upstream source to hide it. `scripts/run-wrf-time-oracle.sh` reproduces
the complete sequence without modifying the pinned source.

The upstream golden output contains 89 named `PASS` arithmetic/formatting cases
plus four clock cases, for 93 active cases total. All 93 names now occur in
executing Rust assertions, verified by
`scripts/check-wrf-time-case-coverage.sh`.

## Last verified commands

```text
cargo fmt --all
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo test --workspace --release
./scripts/check-wrf-time-case-coverage.sh
./scripts/run-wrf-time-oracle.sh
./scripts/run-positive-definite-oracle.sh
./scripts/run-held-suarez-oracle.sh
./scripts/run-column-mass-staggering-oracle.sh
./scripts/benchmark-held-suarez-fortran.sh
./scripts/benchmark-positive-definite-fortran.sh
./scripts/benchmark-column-mass-staggering-fortran.sh
cargo bench -p wrf-dynamics --bench column_mass_staggering -- --noplot
cargo run -p wrf-dynamics --release --example measure_held_suarez_allocations
cargo run -p wrf-dynamics --release --example measure_column_mass_staggering_allocations
```

Result: 52 unit tests and three doctests passed in debug and release modes (11
`wrf-compute`, 21 `wrf-dynamics`, 20 `wrf-time`), including all-target benchmark
smoke execution. Clippy and rustdoc are clean. All 93 active WRF time cases are
referenced by executing Rust assertions, both Fortran time interfaces match
`Test1.out.correct`, both positive-definite oracles match raw IEEE-754 bits, the
16-selection Held-Suarez boundary oracle matches exactly, and all 240
`calc_mu_staggered` output/sentinel bits match across its eight axis/path
combinations. The column-mass matched benchmark, one/four/host-worker Criterion
run, allocation budget, optimized assembly inspection, and rejected SIMD screen
are recorded in the performance ledger and detailed baseline.

## Maintained knowledge and quality ledgers

- `docs/wiki/README.md`: technical encyclopedia and onboarding index.
- `TEST_COVERAGE.md`: upstream, adversarial, concurrency, and operational gaps.
- `UPSTREAM_FINDINGS.md`: reproducible Fortran bugs, test gaps, and performance
  opportunities with confidence labels.
- `PERFORMANCE_PARITY.md`: matched Rust/Fortran workload policy and cumulative
  speed ratios.
- Public crates enable missing-doc warnings and deny broken rustdoc links.

## Git checkpoints

- `bb6cc55` — pinned source tooling, time parity, compute architecture, both
  positive-definite kernels, wiki, coverage ledger, and upstream findings.
- `0ee002d` — Criterion throughput/scaling harness and scalar baseline.
- `7389443` — instrumented steady-state allocation budgets and measurements.
- `5b5f7aa` — documented and removed a parity-correct SIMD prototype that
  regressed representative positive-definite benchmarks.
- `4e6af9a` — nested scientific module hierarchy, Held-Suarez scalar parity,
  matched optimized-Fortran benchmark, wiki, and coverage/findings updates.
- `58bcb67` — accepted safe Held-Suarez SIMD, allocation evidence, scalar/SIMD
  parity corpus, and docs.rs example.
- `d0ec31d` — matched positive-definite Fortran benchmark, optimization-level
  audit, and rejected bench-only native/fat-LTO profiles.
- `8d5e112` — durable state pointer for the benchmark checkpoint.
- `adef46f` — interior ARW column-mass staggering, exact extracted-source
  oracle, typed ranges/errors, concurrency coverage, wiki, and findings.
- `67d9ce3` — all `calc_mu_staggered` physical-boundary paths, domain/tile/memory
  separation, 240-value exact Fortran corpus, and all-boundary determinism.
- `dd3e903` — matched column-mass Criterion/Fortran benchmark harnesses and
  warmed allocation instrumentation.

## Immediate next actions

1. Build a randomized differential corpus for all completed dynamics kernels.
2. Port the WRF Registry DSL and generated-state fixtures.
3. Measure Held-Suarez SIMD on x86-64 when that architecture is available.
