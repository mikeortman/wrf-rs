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
- Stop performance tuning when matched Rust and Fortran implementations are
  operationally close unless a measured model-level bottleneck justifies more
  complexity.

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

### `wrf-registry`

Implemented:

- a safe typed parser for dependency-closed `dimspec`, `state`, and `rconfig`
  entries;
- WRF-compatible backslash continuation, case folding, quoted-token, comment,
  and quoted-`#` behavior;
- physical source locations and typed diagnostics for malformed input;
- typed standard, namelist, and constant dimension bounds, coordinate axes,
  value types, state dimensions/modifiers, staggering flags, time levels, and
  scalar/expression-sized namelist entries;
- source-order resolution of state dimension references;
- a separate selected-artifact generator, with no runtime domain ownership in
  the parser crate;
- byte-identical `state_struct.inc`, `namelist_defines.inc`,
  `namelist_defaults.inc`, `namelist_statements.inc`, and
  `model_data_order.inc` output against WRF v4.7.1;
- exact normalized state metadata for regular time levels and generated
  boundary/boundary-tendency arrays, derived from WRF `allocs_*.F` artifacts;
- committed upstream goldens and a reproducible `scripts/run-registry-oracle.sh`
  differential gate.

The first fixture is deliberately small but uses real ARW `t` and `mu` entry
forms, including continuations, complex I/O specifications, two time levels,
boundary modifiers, every dimension-length mode, and scalar/vector runtime
configuration storage. Includes, conditionals, `typedef`, `i1`, packages,
communication entries, four-dimensional scalar-array generation, and the
remaining generated files are explicitly not yet supported. See
`docs/wiki/WRF-Registry.md`.

### `wrf-domain` and `wrf-domain-mpi`

Implemented:

- signed zero-based half-open domain indices with checked Fortran conversion;
- typed physical domain, horizontal, patch, memory, and tile bounds;
- exact centered-remainder RSL_LITE decomposition and row-major patch IDs;
- WRF guard-point memory bounds with explicit physical-boundary storage;
- edge-tile halo extension followed by physical-domain clipping;
- transport-neutral Y-then-X halo plans with internal corners, periodic
  endpoints, and per-axis field staggering;
- deterministic local execution using boundary-sized staged messages;
- one-patch XZY storage for rank-local transport without field-sized clones;
- a separate safe MPI adapter using receives-before-sends non-blocking phases;
- direct pinned `task_for_point.c` and `period.c`/`f_pack.F90` differential
  oracles; and
- complete four-rank MPI versus local-memory parity for nonperiodic and doubly
  periodic staggered cases.

Topology is setup work and no misleading throughput ratio is recorded. A
matched halo benchmark waits for WRF-compatible multi-field aggregation.

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
  one/four-worker determinism tests at all physical boundaries;
- a versioned SplitMix64 corpus generator, shared raw-bit input files, and
  pinned Fortran drivers for 68 seeded cases and 39,588 complete outputs across
  all four translated routines;
- exact finite/infinity output comparison, explicit NaN-class policy, and
  first-divergence reports containing seed, field, and linear index.

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
focused and seeded corpora. Interior subdomain tiles use halo mass points;
physical lower and upper boundaries copy the nearest full mass exactly as WRF
does. Its 16 randomized cases cross all four boundary states on both axes. The
next ARW gates are periodic `calc_mu_uv` variants and idealized-case integration.

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
- Matched Fortran performance is a stopping signal as well as an optimization
  signal. If portable Rust is already close, the standard multithreaded path is
  competitive, allocations are bounded, and no full-model profile shows a
  material hotspot, record the result and move on. Do not spend port time on a
  fragile marginal win.

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

## Seeded randomized ARW parity

`tools/arw-corpus-generator` produces committed language-neutral inputs for all
translated dynamics routines. The corpus contains 24 sheet cases, 16 slab
cases, 12 Held-Suarez cases, and 16 column-mass cases. It varies small shapes,
negative and non-one memory origins, domain/tile clipping, signed zero, finite
magnitude extremes, and active NaN/infinity values. The column-mass cases cover
all 16 cross-axis physical-boundary combinations.

`scripts/randomized-arw/run-oracles.sh` regenerates and byte-compares the input
files before compiling the pinned WRF routines. Rust consumes those same inputs
and all 39,588 Fortran-derived output records. Finite values, signed zero, and
infinities match raw bits; NaN matches by class because payload propagation is
not portable. Current default-host-parallel Rust passes every case.

Finite extreme sheet seed `1695930` and slab seed `2771003` reproduce
intermediate multiplication overflow in WRF's normalization. Rust preserves the
infinity results; `UPSTREAM_FINDINGS.md` records this as WRF-008 rather than
silently changing operation order.

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
./scripts/randomized-arw/run-oracles.sh
./scripts/run-registry-oracle.sh
./scripts/run-domain-topology-oracle.sh
./scripts/run-clipped-tiles-oracle.sh
./scripts/run-mpi-halo-parity.sh
./scripts/run-periodic-halo-oracle.sh
./scripts/benchmark-held-suarez-fortran.sh
./scripts/benchmark-positive-definite-fortran.sh
./scripts/benchmark-column-mass-staggering-fortran.sh
cargo bench -p wrf-dynamics --bench column_mass_staggering -- --noplot
cargo run -p wrf-dynamics --release --example measure_held_suarez_allocations
cargo run -p wrf-dynamics --release --example measure_column_mass_staggering_allocations
```

Result: 89 unit tests and four doctests passed in debug and release modes (one
corpus-generator test, 11 `wrf-compute`, 15 `wrf-domain`, two
`wrf-domain-mpi`, 25 `wrf-dynamics`, 15 `wrf-registry`, and 20 `wrf-time`),
including all-target benchmark smoke execution. Clippy and rustdoc are clean.
All 93 active WRF time cases are referenced by executing Rust assertions, both
Fortran time interfaces match `Test1.out.correct`, the focused numerical
oracles remain exact, and all four randomized Fortran corpora pass their 39,588
complete-output comparisons. The column-mass matched benchmark,
one/four/host-worker Criterion run, allocation budget, optimized assembly
inspection, and rejected SIMD screen remain recorded in the performance ledger
and detailed baseline. The WRF Registry oracle matches five generated includes
and eight state-metadata records exactly. Domain decomposition and clipped
tiles match pinned WRF routines, periodic destinations match `period.c`
exactly, and complete four-rank MPI patch memory matches the local executor.

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
- `f6dd8e6` — versioned seeded ARW input generator, four pinned Fortran corpus
  drivers, 39,588 complete-output comparisons, CI gate, and exceptional-value
  policy in rustdoc.
- `dcb30e3` — typed Registry parser, selected exact artifact generator, upstream
  fixture/goldens, malformed-input coverage, and CI oracle.
- `076caa1` — Registry architecture, language inventory, wiki, state ledgers,
  and confirmed upstream allocation-generator finding.

## Immediate next actions

1. Add Registry-generated asymmetric halo descriptors and multi-field message
   aggregation to the domain transport.
2. Extend Registry preprocessing with includes and conditional definitions.
3. Add periodic `calc_mu_uv` parity before larger ARW coupling work.
4. Add Registry packages, typedefs, communication entries, and remaining
   generated artifacts in dependency-closed slices.
5. Measure Held-Suarez SIMD on x86-64 when that architecture is available.
