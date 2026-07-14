# Repository instructions

These instructions apply to the whole repository. More specific `AGENTS.md`
files may tighten them for a subtree, but may not weaken a repository rule.

## Canonical contract

This file is the repository's self-contained operating contract. It includes
the Rust backend rules from [`RUST_BACKEND_STYLE_GUIDE.md`](RUST_BACKEND_STYLE_GUIDE.md)
and the high-performance Rust/Fortran guidance used for this port.

**MUST follow [`RUST_BACKEND_STYLE_GUIDE.md`](RUST_BACKEND_STYLE_GUIDE.md) for
all Rust design, implementation, review, testing, and documentation work.**
The standalone file contains the longer reference examples; the compatible
rules are inlined below so an agent can work correctly from this file alone.
When a general recommendation conflicts with a WRF-specific rule here, the
WRF-specific rule wins. Ask before making an exception.

## Sources of truth

- GitHub Issues, sub-issues, and the **WRF Rust Port** Project own mutable work
  state, priorities, dependencies, and long-horizon plans.
- `tracking/port-status.json` owns the current coarse parity state.
- `tracking/benchmarks.json` owns matched benchmark definitions and change
  routing.
- `docs/generated/` is generated. Run `python3 tools/tracking.py render`; never
  hand-edit generated files.
- GitHub Actions benchmark artifacts, job summaries, and merged-PR receipt
  comments own measurements from CI. Detailed `docs/performance/` pages remain
  immutable engineering records, not the current-value database.
- `CURRENT_STATE.md`, `TEST_COVERAGE.md`, and `PERFORMANCE_PARITY.md` are legacy
  narrative references. Do not add mutable totals, TODO lists, or benchmark
  tables to them.

Start work by querying GitHub, not by reconstructing a backlog from Markdown:

```sh
gh issue list --repo mikeortman/wrf-rs --state open --limit 100
gh project item-list 1 --owner mikeortman
```

## Change workflow

1. Every dependency-closed slice starts with one issue. Use parent issues and
   sub-issues for objectives that span several independently mergeable slices.
   Add active issues to the **WRF Rust Port** Project and set Area, Work type,
   Horizon, and Evidence gate. Repository issue forms add the Project
   automatically; CLI-created issues should use `--project "WRF Rust Port"`.
2. Fetch `main`, branch from the fetched tip as
   `codex/issue-N-short-name`, and do not stack a new branch on an unmerged
   pull request.
3. Keep one issue-linked pull request per slice. Its body must close the issue.
4. Enable auto-merge only after required checks are present. Until this
   user-owned repository can use GitHub's native merge queue, the strict
   up-to-date branch rule and unstacked branches are the queue substitute.
5. Commit focused checkpoints. Preserve unrelated user changes.

## Scientific acceptance gates

- Pin the exact WRF source routine or upstream regression test before porting.
- Preserve discrete behavior. Use exact bits by default; any ULP or tolerance
  policy must be explicit, justified, and tested at intermediate states.
- Add a direct Fortran oracle, deterministic fixture, malformed-input tests,
  boundary cases, and seeded randomized coverage where useful.
- Test failure atomicity, one-worker behavior, and multithreaded determinism.
- A slice is not complete because Rust-only tests pass.

## Rust backend contract

The following rules are the inlined form of the standalone style guide. The
severity words are normative:

- `MUST` is a blocking requirement.
- `SHOULD` is the default unless a clear local reason is recorded.
- `CONSIDER` is optional and evidence-driven.
- `AVOID` requires a specific justification.

### Structure and ownership

- **MUST** organize code around scientific or domain ownership, not a flat
  collection of unrelated files.
- **MUST** use one primary `struct`, `enum`, or trait per file by default. When
  a file grows, split it into a module directory with a focused `mod.rs`
  facade.
- **MUST** preserve public import paths during refactors with deliberate
  re-exports.
- **SHOULD** use thematic subfolders and nested submodules for large systems.
- **AVOID** mixing unrelated public concerns or leaving business logic without
  an honest owning type or capability boundary.
- **MUST** keep implementation details private or `pub(crate)` by default;
  expose `pub` only for a real external contract.

### Names and domain types

- **MUST** use `PascalCase` for types and traits and `snake_case` for functions,
  methods, variables, fields, files, and directories.
- **MUST** prefer explicit names over abbreviations. Standard abbreviations such
  as `id`, `url`, `uri`, `http`, `json`, and `api` are acceptable; public names
  such as `Ctx`, `Cfg`, `Mgr`, `Svc`, `Req`, and `Resp` are not.
- **MUST** use meaningful `XxxId` newtypes in public interfaces instead of raw
  integer identifiers. Derive the common comparison and hashing traits that
  the domain needs.
- **SHOULD** use predicate names such as `is_`, `has_`, `can_`, `needs_`, and
  `should_`, and consistent `new`, `try_new`, `from_*`, `try_from_*`, `as_*`,
  `into_*`, and `with_*` names.
- **SHOULD** make collection names reveal their shape and keying, such as
  `items_by_id` or `pending_job_ids`.

### Functions, traits, and errors

- **MUST** keep functions narrow, single-purpose, and at one level of
  abstraction. Prefer associated methods or focused capability traits when a
  real owner exists; keep a free function only when no owner would be honest.
- **MUST** use guard clauses and named helpers to keep branch depth shallow;
  prefer `match` for finite state sets and avoid more than four nested branch
  levels in production code.
- **MUST** use `Result` for recoverable failures and typed error enums at
  service, API, worker, adapter, and persistence boundaries. Major boundaries
  should expose typed result aliases and semantic variants such as `NotFound`,
  `InvalidInput`, `Conflict`, `Unavailable`, and `Internal`.
- **AVOID** raw `String` errors, stable public `Box<dyn Error>` channels,
  `unwrap` or `expect` in normal runtime flow, hidden retries, and ad hoc async
  runtimes.
- **MUST** keep each trait cohesive, document its contract and failure modes,
  and use `Send + Sync` only when cross-thread use is intended.
- **SHOULD** keep timeout, cancellation, retry bounds, queue bounds, and fan-out
  explicit. Do not block inside async code.

### Imports, visibility, docs, and tests

- **MUST** keep imports at the top, import symbols explicitly, and use the
  imported names at call sites. Remove unused imports and avoid wildcards
  outside controlled test preludes.
- **MUST** add useful rustdoc for public types, traits, and methods. Explain
  intent, units, equations, indexing, invariants, provenance, and failure
  behavior where relevant; comments must not compensate for unclear names.
- **MUST** put ordinary unit tests in the same source file, at the bottom, in a
  `#[cfg(test)] mod tests` module. Name tests by behavior and expected result.
- **SHOULD** cover happy paths, edge dimensions, malformed input, failures,
  determinism, and representative randomized cases. Keep fixtures close to the
  tests.
- **MUST** use staged, named transformations for parsing, validation,
  normalization, enrichment, persistence, and computation. Keep mutation local
  and short-lived; avoid dense nested chains.

### Review and generation behavior

When reviewing Rust, report `MUST` violations first and tie findings to a rule,
file, symbol, severity, and concrete correction. Do not demand optional
refactors when the code already satisfies the contract. When generating code,
prefer the smallest clear implementation, add behavior-focused tests, and state
assumptions. Stop when the requested artifact is complete or a missing decision
materially blocks it.

## Performance and Fortran parity contract

Treat optimized Fortran as a serious baseline. The goal is numerically
equivalent, maintainable Rust with predictable machine behavior, not a textual
translation or an unverified claim that Rust is faster. Dense kernels often
reach parity; Rust's strongest opportunities are better whole-program layout,
fewer copies, safer coarse-grained parallelism, sparse/irregular data, and
better integration around the kernel.

### Fair builds and numerical semantics

- Benchmark release-like builds only. Record Rust version, LLVM version, target
  CPU flags, Cargo profile, compiler flags, thread count, CPU, cores, memory,
  operating system, input dimensions, and iteration count.
- The default Rust performance profile is `opt-level = 3`, LTO as appropriate,
  and minimal codegen units. Use `RUSTFLAGS="-C target-cpu=native"` only for
  machine-specific results and label the binary as non-portable.
- Compare against an equivalently optimized Fortran build, commonly
  `gfortran -O3 -march=native -flto`, with the same workload, thread count,
  warm-up, cache assumptions, and setup costs. Use the actual WRF compiler
  configuration when that is the production baseline.
- Do not compare relaxed floating-point Fortran with strict Rust and attribute
  differences to the language. Do not enable `-ffast-math` or equivalent
  semantics silently. If relaxed math or a precision change is necessary,
  document NaN, infinity, signed-zero, rounding, reproducibility, and absolute
  and relative error policies, then add reference-value and pathological-input
  tests.

### Correctness before optimization

- Establish a trusted Fortran oracle before changing performance behavior.
- Test hand-verifiable small cases, minimum and non-SIMD dimensions, halo and
  boundary regions, randomized inputs, malformed inputs, NaN/infinity behavior
  where relevant, and serial versus parallel results.
- Separate scalar correctness, layout changes, allocation removal, bounds or
  compiler improvements, auto-vectorization, parallelism, and any SIMD or
  accelerator work into independently testable slices.
- Treat reproducibility explicitly: record whether a result is bitwise,
  tolerance-based, order-dependent, thread-count-dependent, or CPU-feature-
  dependent. A faster result that is not scientifically equivalent is a bug.

### Layout, ownership, and allocation

- **MUST** use deliberate contiguous storage for dense fields. Identify whether
  the Rust representation is row-major or another order and keep the
  contiguous dimension innermost; do not mechanically preserve Fortran loop
  order when it harms locality.
- Consider structure-of-arrays when kernels touch only a few attributes, and
  array-of-structures when each element's fields are consumed together. Choose
  from measured access patterns.
- Express non-aliasing with safe `&mut`/slice boundaries, `chunks_exact`, and
  `split_at_mut` where practical. Prefer separate input/output buffers and
  double buffering for stencils and timesteps.
- **MUST NOT** allocate, log, lock, format errors, or make large clones inside a
  hot loop. Preallocate reusable workspaces, borrow read-only views, swap
  buffers, and separate allocation, initialization, kernel, validation, and
  deallocation costs in benchmarks.
- Do not use uninitialized memory or pointer arithmetic merely to improve a
  benchmark. This repository forbids handwritten `unsafe` Rust. The general
  style guide's low-level `unsafe` examples do not override that rule.

### Vectorization and numerical kernels

- Write simple, contiguous, independent loops with explicit aliasing and no
  hidden dispatch. Verify bounds-check removal, inlining, and vectorization in
  generated code for foundational kernels instead of assuming that an iterator
  or attribute optimized away.
- Use maintained, trusted **safe** SIMD crates only when profiling shows a
  confirmed bottleneck, auto-vectorization is inadequate, supported targets are
  known, scalar fallback exists, empty and tail lengths are covered, and parity
  tests pass. Do not keep complex SIMD or target-specific tuning for a small or
  noisy gain.
- Precompute invariant coefficients and fuse passes only when memory traffic is
  the bottleneck and measured evidence shows the fused form helps without
  increasing register pressure or harming scheduling.
- Avoid dynamic dispatch, virtual calls, per-element validation, logging, rich
  error construction, and unpredictable branches in critical loops. Do not
  replace clear branches with complicated branchless code without measurements.
- Use approximations, reduced precision, vector math libraries, FFI, or GPU
  offload only with error analysis, boundary tests, ownership of the numerical
  contract, and end-to-end evidence. Keep future accelerators behind narrow
  capability boundaries; do not move data to a device for one tiny operation.

### Parallelism and system behavior

- Prefer coarse-grained parallel work over per-element tasks. Measure scaling
  across deliberate thread counts and prevent Rayon, BLAS, MPI, and application
  runtimes from oversubscribing one another.
- Use disjoint contiguous output, per-thread state, or batched merging instead
  of locks in numerical loops. Consider false sharing, NUMA placement, and
  deterministic reduction order.
- For distributed work, overlap communication with independent interior work,
  then process boundary regions and reductions after synchronization. Use
  contiguous typed buffers for regular halo exchange rather than serializing
  Rust object graphs.

### Benchmark, profile, and report

- Begin with a representative production workload, warm up, run multiple
  samples, prevent dead-code elimination, and report medians plus variability.
  Measure both isolated kernels and complete timesteps or applications.
- Profile before low-level changes. Determine whether the bottleneck is
  arithmetic, memory bandwidth, allocation, synchronization, communication,
  I/O, or the wrong loop. Inspect generated assembly or LLVM for critical
  kernels when claiming vectorization or eliminated calls.
- Use this optimization order unless evidence says otherwise: fair release
  build, correctness, end-to-end measurement, profiling, layout and loop order,
  allocation and copy removal, boundary/interior separation, alias visibility,
  compiler checks, measured fusion or tiling, coarse parallelism, affinity and
  communication, then explicit safe SIMD or GPU work.
- A performance receipt must include workload, hardware, compiler and flags,
  thread count, numerical error, operation-order differences, Fortran median,
  Rust scalar/SIMD/parallel medians when present, speedups, variability, and the
  primary remaining bottleneck. Never publish a single best-case run.

## Definition of done

A porting slice is complete only when its issue-linked change is readable,
documented, tested against the Fortran oracle, and covered by the applicable
Rust and Fortran checks. For performance-sensitive work, the benchmark is
reproducible, the compiler behavior is evidenced, the numerical semantics are
documented, and any improvement has an understood cause. Treat close Rust and
Fortran results as success; pursue additional tuning only when the measured
gain is material and does not reduce clarity, correctness, or portability.

Before requesting merge, run the applicable subset and then the full gates:

```sh
python3 tools/tracking.py check
cargo fmt --all --check
cargo test --workspace --all-targets
cargo test --workspace --release
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Run every affected Fortran oracle. Review `git diff --check` and the complete
diff before committing.
