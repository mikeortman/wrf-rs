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

## Detailed high-performance guidance

The short contract above is the decision rule. This section preserves the
technical detail needed to apply it correctly.

### Fair compiler configuration

Use equivalent optimized configurations before drawing a language conclusion.
For a machine-specific Rust measurement, a benchmark profile may use:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
debug = false
panic = "abort"
```

Record every non-default choice. A typical local-only command is:

```sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

For GNU Fortran, the comparable starting point is commonly:

```sh
gfortran -O3 -march=native -flto source.f90 -o benchmark-fortran
```

Use the production WRF compiler and flags when they are the actual baseline.
Do not compare Rust release code with debug Fortran, native Rust with generic
Fortran, one thread against many, warm cache against cold cache, or different
algorithms while calling the result a language comparison. `target-cpu=native`
produces a binary that may not run on older processors.

### Floating-point semantics

Flags such as `-ffast-math` can reassociate expressions, use fused operations,
ignore signed zero, change NaN and infinity handling, and make reduction order
non-deterministic. They can be useful, but they are a numerical-semantic change,
not a harmless optimization.

Before relaxing floating-point behavior:

- Define acceptable absolute and relative error.
- Add reference-value tests and pathological-input tests.
- Decide whether NaN, infinity, denormals, and signed zero matter.
- Validate the result on every supported CPU class.
- Record whether operation order and reproducibility changed.

Never silently mix relaxed Fortran semantics with strict Rust semantics.

### Benchmark protocol

A valid benchmark represents the real workload, not a convenient toy loop. It
must use release builds, realistic dimensions, representative sparsity and
branch behavior, and enough iterations to reduce timer noise. Include a warm-up
phase, multiple samples, dead-code prevention, and a stable input seed.

Measure and report separately:

- Allocation and deallocation.
- Initialization and zero-filling.
- Kernel execution.
- Result validation and serialization.
- Complete timestep and end-to-end application runtime.

At minimum record CPU model, physical and logical cores, memory, operating
system, Rust and LLVM versions, Fortran compiler and version, compiler flags,
thread count, input dimensions, iteration count, median and minimum runtime,
throughput, and numerical error. Criterion or an equivalent harness is
appropriate for Rust microbenchmarks. Use the same cache, affinity, input, and
thread conditions for both languages.

### Correctness before optimization

Create the trusted Fortran oracle before changing the algorithm. Establish a
small hand-verifiable fixture, then cover randomized inputs, minimum dimensions,
dimensions that are not multiples of a vector width, halo and boundary regions,
malformed input, NaN and infinity behavior where relevant, and serial versus
parallel execution. Compare intermediate states when the final output alone
could hide a divergence.

Keep these stages separately reviewable and benchmarkable:

1. Scalar correctness port.
2. Layout correction.
3. Allocation and copy removal.
4. Loop and bounds-check improvements.
5. Auto-vectorization improvements.
6. Safe explicit SIMD, only if justified.
7. Shared-memory parallelism.
8. Distributed-memory or accelerator work.

Do not combine a language port, algorithm change, precision change, parallel
rewrite, and SIMD rewrite into one unreviewable change.

### Flat storage and memory order

Dense multidimensional fields should normally use one contiguous allocation.
Nested `Vec<Vec<T>>` storage adds allocations, pointers, metadata, poor
prefetching, and harder vectorization; use it only when rows genuinely have
different lengths or independent lifetimes.

For every field, write down the storage order and answer:

- Which index is contiguous?
- Which loop varies that index?
- Are neighboring threads writing neighboring cache lines?
- Are halo accesses strided?
- Can rows, planes, or tiles be processed as a unit?

Fortran arrays are conventionally column-major, with the first index varying
fastest. A Rust row-major representation should usually keep the x or row
offset contiguous and innermost. Preserve the numerical mapping, not the source
loop order, when the representation changes. A poor loop order can multiply
cache-line loads and TLB pressure while preventing vectorization.

### Structure of arrays and array of structures

Use structure-of-arrays when a kernel touches one or a few attributes at a time:
separate contiguous density, pressure, temperature, and velocity arrays can
improve SIMD loads, cache capacity, bandwidth, compression, and device
interoperability. Use array-of-structures when nearly every operation consumes
all fields for one element. Benchmark the real access pattern; neither layout
is universally superior.

### Aliasing, borrowing, and bounds

Prefer `&mut [T]` for an output and `&[T]` for read-only inputs. Rust's exclusive
mutable reference communicates non-aliasing to the compiler without raw
pointers. Validate lengths once before a loop, then use slices, iterators,
`chunks_exact`, or row views so LLVM can prove bounds.

When two mutable regions are disjoint, express that fact with safe slicing such
as `split_at_mut`. For stencils, separate input and output buffers when
possible; double buffering is clearer and often easier to vectorize than a
complex in-place update.

This repository does not permit `unsafe`, `get_unchecked`, raw pointers, or
manual pointer arithmetic as a bounds-check escape hatch. If a safe loop appears
to retain a real check in a measured hot path, first change the layout or loop
shape and inspect generated code. A trusted safe crate may encapsulate low-level
implementation details, but it must not weaken the public safety contract.

### Allocation and copy policy

Never allocate a temporary field inside a timestep or iteration loop. Store
scratch arrays, residuals, neighbor lists, masks, and communication buffers in a
reusable workspace. Reuse them with `fill`, and swap current and next buffers
instead of cloning or copying a full field. Accept a lightweight clone only
when it materially improves ownership clarity and is not a large numerical
allocation.

Do not accept an owned `Vec<T>` when a borrowed slice is sufficient. Distinguish
allocation, initialization, kernel, validation, and deallocation in benchmarks;
initializing a large zeroed vector can dominate a short kernel. Do not use
uninitialized storage merely to improve a benchmark unless a reviewed safe
abstraction proves that every element is written before it is read.

### Auto-vectorization and inlining

LLVM is most likely to vectorize loops with contiguous or fixed-stride accesses,
independent iterations, simple arithmetic, no allocation, no virtual dispatch,
no hidden calls, no unpredictable exits, and clear aliasing. Reductions need an
explicitly understood order and error policy.

Use `#[inline]` sparingly for tiny indexing helpers, small arithmetic called in
hot loops, generic wrappers that should disappear, and light abstraction
boundaries. Do not mark large functions `#[inline(always)]` by default; excessive
inlining can grow the binary, harm instruction-cache behavior, slow compilation,
and make assembly harder to read.

Generic static dispatch may optimize away, but a closure with complex control
flow can still inhibit vectorization. Dynamic dispatch should not remain inside
a foundational numerical loop unless profiling proves it irrelevant.

### Safe SIMD

Use explicit SIMD only after profiling identifies a confirmed bottleneck and
generated code shows that auto-vectorization is inadequate. Prefer maintained,
trusted safe SIMD crates over local architecture intrinsics. Require a scalar
fallback, known supported targets, parity tests, representative vector sizes,
empty inputs, non-multiple tail lengths, and a benchmark that includes dispatch
overhead. Use unaligned access unless alignment is proven by a safe container.

Do not assume wider vectors are faster: AVX-512 can lower CPU frequency and
memory-bound kernels may gain nothing from more arithmetic width. Remove a SIMD
path that is not measurably faster or that makes the code materially harder to
understand. Never trade exact numerical behavior for a small noisy gain.

### Reductions, stencils, and boundaries

Reduction order affects floating-point results. Mark each reduction as bitwise
reproducible, tolerance-based, thread-count-dependent, CPU-feature-dependent, or
order-dependent. Use deterministic tree or fixed-chunk reductions when the
scientific contract requires it, and test serial and parallel results.

Separate regular interior work from boundary and halo work when that improves
vectorization or branch predictability. Keep boundary conditions explicit and
test minimum grids, one-cell grids, halo widths, and dimensions near tile or
SIMD boundaries. Do not apply an interior stencil to a boundary merely to make
the loop look uniform.

### Cache blocking and tiling

Tiling can improve reuse when a working set does not fit in cache. Choose tiles
from measured access patterns and account for halos, write allocation, and
thread ownership. Benchmark several tile sizes and compare the untiled version;
tiling can hurt when it adds index arithmetic, register pressure, or scheduling
overhead. Keep a clear row or plane abstraction rather than introducing opaque
pointer math.

### Shared-memory parallelism

Use coarse tasks with enough work to amortize scheduling. Measure scaling at
several thread counts, including one worker, and control Rayon, BLAS, MPI, and
application thread pools so they do not oversubscribe the same cores. Prefer
disjoint output slices, per-thread buffers, and one batched merge over locks in a
numerical hot loop. Atomics are appropriate for genuinely low-contention
counters, not repeated field updates.

Consider false sharing when independent values share cache lines. Consider
NUMA placement for large fields and record thread affinity and placement in the
benchmark receipt. A parallel result must be tested for determinism and parity,
not only for elapsed time.

### Distributed memory and MPI

For regular halo exchange, use a staged sequence when the dependency graph
permits it: start halo exchange, compute independent interior cells, wait for
completion, compute boundary-adjacent cells, perform reductions, and advance
buffers. Keep communication buffers contiguous and typed. Do not serialize a
large Rust object graph for every halo exchange when a direct field buffer is
available. Measure communication, synchronization, and complete timestep time,
not just the local kernel.

### FFI, sparse work, and precision

Use existing optimized numerical libraries through narrow, documented safe
wrappers when they are the right fit. Make ownership, layout, error conversion,
threading, and numerical semantics explicit at the boundary. FFI is not a reason
to hide an unmeasured copy or to bypass the Rust safety contract.

Sparse and irregular kernels may benefit more from compact indices, worklists,
specialized storage, or scheduling than from dense SIMD. Measure the actual
branch and access distribution. For precision changes, consider storing fields
in `f32` while accumulating sensitive reductions or solver steps in `f64`, but
convert only at defined boundaries and validate stability, iteration count,
dynamic range, and physical behavior.

### Branches, calls, errors, and logging

Unpredictable branches can block vectorization. Consider partitioning common and
rare cases, processing masks, or moving rare handling out of the main loop, but
do not replace a clear predictable branch with complicated branchless arithmetic
without evidence.

Validate dimensions and configuration once at a public entry point. Keep the
inner kernel infallible where possible; do not allocate rich errors or format
messages for every element. Do not log per cell, particle, iteration, or grid
point. Record aggregate metrics such as residual norm, cells processed, bytes
transferred, halo duration, iteration count, and maximum error, preferably behind
a compile-time or feature-gated instrumentation path.

### GPU offload

GPU work is justified when data parallelism and arithmetic intensity are high,
working sets remain device-resident across multiple operations, transfers are
limited, launch overhead is amortized, and branch divergence is manageable.
Benchmark host-to-device transfer, kernel execution, synchronization,
device-to-host transfer, complete timestep, and complete application. A fast
isolated kernel that requires a full-field copy every step is not an end-to-end
win.

### Assembly and profiling

For foundational kernels, inspect generated assembly or LLVM IR when claiming an
optimization. Look for scalar versus vector instructions, repeated bounds
branches, calls in the loop, redundant loads and stores, stack spills, failed
inlining, expensive conversions, inner-loop division, gather/scatter, hidden
copies, and panic paths. `cargo asm` or an equivalent tool can identify the
monomorphized symbol.

Profile the complete release binary before rewriting code. Useful questions are
whether the program is compute-bound, memory-bandwidth-bound, allocation-bound,
synchronization-bound, communication-bound, or I/O-bound. Useful counters
include cycles, instructions, branches, branch-misses, cache references, cache
misses, page faults, and context switches. Optimize the measured bottleneck,
not the most mathematically interesting function.

### Bandwidth, fusion, and precomputation

Estimate arithmetic intensity as floating-point operations divided by bytes
transferred. An operation that reads two `f64` values and writes one transfers
roughly 24 bytes per element before cache reuse, so extra SIMD may provide less
benefit than fewer passes, kernel fusion, blocking, or reduced precision.

Fuse compatible passes when the intermediate need not be stored, the combined
loop still vectorizes, register pressure stays reasonable, and scheduling is
not harmed. Fusion can be slower when it causes spills, mixes access patterns,
or prevents cache reuse. Benchmark fused and unfused forms.

Move invariant divisions, reciprocals, coefficients, geometry terms, masks,
neighbor lists, lookup tables, and communication schedules outside loops when
the memory cost is justified. Do not precompute a table so large that it evicts
the data it was meant to accelerate.

### Index arithmetic, math functions, iterators, and abstractions

Row and plane slicing can remove repeated multidimensional index arithmetic and
make locality obvious. Do not replace every index calculation with pointer-like
manual arithmetic unless generated code proves integer arithmetic is the
bottleneck. Precompute reciprocals for repeated division when numerical error
permits. Treat square root, exponential, logarithmic, and trigonometric calls as
potential bottlenecks; approximations require error analysis, boundary tests,
documentation, and scientific-owner approval.

Idiomatic iterators often compile to excellent loops. Reject only iterator forms
that allocate intermediate collections, box closures, use dynamic dispatch,
obscure locality, or make generated-code inspection impractical. Zero-cost
abstraction is an outcome to verify: hot-path abstractions should preserve
contiguous layout, alias information, monomorphization, inlining, and the
absence of allocation, locking, and hidden reference counting. Do not use
`Arc<Mutex<Vec<T>>>` as the default representation of parallel numerical data.

### Common performance failures

When a result is unexpectedly slow, check these failure modes before inventing a
new algorithm:

- Debug build used instead of release.
- Nested vectors or wrong loop order damaging locality.
- Allocation, zero-filling, cloning, or copying inside iterations.
- Dynamic dispatch or hidden calls in a hot loop.
- Parallel tasks too small, false sharing, or oversubscribed runtimes.
- Bounds checks or scalar arithmetic that remain after layout mistakes.
- Missing vectorization despite regular data.
- Initialization included in a kernel-only claim.
- Numerical mismatch caused by precision, fast-math, or reduction order.

### Detailed performance review checklist

Before claiming parity or a speedup, confirm:

- Reference outputs and documented tolerances match, including edge dimensions,
  tails, parallel results, and special values where relevant.
- Rust and Fortran are release-like, CPU flags and floating-point semantics are
  comparable, and all setup costs are identified.
- Dense fields are contiguous, loop order matches layout, workspaces are reused,
  large clones are justified, and alignment assumptions are explicit.
- Critical helpers, bounds checks, vectorization, dynamic dispatch, and generated
  instructions were inspected rather than assumed.
- Task granularity, thread counts, false sharing, NUMA placement, and nested
  runtime oversubscription were considered.
- The workload is representative, warmed up, sampled repeatedly, protected from
  dead-code elimination, and measured both in isolation and end to end.

The performance report should include:

- Workload and dimensions.
- Hardware, physical and logical cores, memory, operating system, and affinity.
- Rust version, LLVM version, Cargo profile, `RUSTFLAGS`, and thread count.
- Fortran compiler, version, flags, and thread count.
- Maximum absolute and relative error and whether operation order differs.
- Fortran, Rust scalar, safe SIMD, and Rust parallel median runtimes when used.
- Variability, speedup relative to Fortran, memory bandwidth, cache misses,
  vector width, and the primary remaining bottleneck.
- CPU-specific binary, relaxed floating-point settings, NUMA placement, and any
  excluded setup costs.

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
