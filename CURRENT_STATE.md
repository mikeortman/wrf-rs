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

The differential drivers compile the pinned upstream Fortran module directly.
The sheet covers nine exact-bit cases, including signed zero and the epsilon
branch. The slab fixture covers non-one memory origins, domain/tile clipping,
correction branches, and unchanged halo/stagger sentinels. Rust also proves
sheet bitwise determinism between one and four workers.

The Held-Suarez differential fixture checks 16 active and boundary values with
non-one memory origins. Added Rust tests cover one/four-worker determinism,
shape failure before mutation, all range categories, staggered predecessors,
and the pressure reference level.

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
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo test --workspace --release
./scripts/check-wrf-time-case-coverage.sh
./scripts/run-wrf-time-oracle.sh
./scripts/run-positive-definite-oracle.sh
./scripts/run-held-suarez-oracle.sh
./scripts/benchmark-held-suarez-fortran.sh
cargo run -p wrf-dynamics --release --example measure_held_suarez_allocations
```

Result: 45 unit tests and two doctests passed in debug and release modes (11
`wrf-compute`, 14 `wrf-dynamics`, 20 `wrf-time`), including all-target benchmark
smoke execution. Clippy and rustdoc are clean. The release gate exposed and
fixed a scheduler-test assumption that a small workload would necessarily be
stolen by multiple workers; the test now coordinates overlapping tasks before
asserting concurrency. All 93 active WRF time cases are referenced by executing
Rust assertions, both Fortran time interfaces match `Test1.out.correct`, both
positive-definite oracles match raw IEEE-754 bits, and the 16-selection
Held-Suarez boundary oracle matches exactly.

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

## Immediate next actions

1. Backfill matched Fortran baselines for both positive-definite variants.
2. Measure Held-Suarez SIMD on x86-64 when that architecture is available.
3. Select the next dependency-closed ARW numerical kernel using the same
   oracle, adversarial-test, wiki, rustdoc, findings, and performance workflow.
