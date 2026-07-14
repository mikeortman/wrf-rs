# Repository instructions

These instructions apply to the whole repository. More specific `AGENTS.md`
files may tighten them for a subtree, but may not weaken a repository rule.

## Canonical contract

This file is the repository's self-contained operating contract. It includes
the complete Rust backend style guide and the high-performance Rust/Fortran
guidance used for this port. The style guide is inlined below so this file is
the only instruction source an agent needs.

**MUST follow the Rust backend style contract in this file for all Rust design,
implementation, review, testing, and documentation work.** The repository
specific rules below resolve any general recommendation that does not fit WRF.
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
  compiler improvements, auto-vectorization, parallelism, and SIMD work into
  independently testable slices.
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
- Use approximations, reduced precision, vector math libraries, or FFI only with
  error analysis, boundary tests, ownership of the numerical contract, and
  end-to-end evidence.

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
  communication, then explicit safe SIMD work.
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
8. Distributed-memory work.

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

### Performance examples

These examples are patterns, not copy-and-paste APIs. Adapt the error types,
field types, and numerical semantics to the owning WRF subsystem.

#### Contiguous fields and row-major traversal

Keep dense storage flat and expose row views so the contiguous dimension stays
innermost:

```rust
pub struct Field2D {
    nx: usize,
    values: Vec<f64>,
}

impl Field2D {
    pub fn row(&self, y: usize) -> &[f64] {
        let start = y * self.nx;
        &self.values[start..start + self.nx]
    }

    pub fn row_mut(&mut self, y: usize) -> &mut [f64] {
        let start = y * self.nx;
        &mut self.values[start..start + self.nx]
    }
}

for (output_row, input_row) in output
    .chunks_exact_mut(nx)
    .zip(input.chunks_exact(nx))
{
    for (output_value, &input_value) in output_row.iter_mut().zip(input_row) {
        *output_value += input_value;
    }
}
```

For a Fortran column-major field, preserve the physical mapping deliberately;
do not copy the original loop nesting until the Rust storage order is decided.

#### Safe aliasing and validated kernels

Validate shape once at the public boundary, then keep the inner loop simple:

```rust
pub fn axpy(
    destination: &mut [f64],
    source: &[f64],
    scale: f64,
) -> Result<(), KernelError> {
    if destination.len() != source.len() {
        return Err(KernelError::LengthMismatch);
    }

    for (destination_value, &source_value) in destination.iter_mut().zip(source) {
        *destination_value += scale * source_value;
    }

    Ok(())
}
```

The exclusive `&mut` output and shared input communicate non-aliasing without
raw pointers. A kernel needing two disjoint mutable regions should make that
fact visible with safe slicing:

```rust
let (left, right) = values.split_at_mut(split);
for value in left {
    *value *= 2.0;
}
for value in right {
    *value *= 3.0;
}
```

#### Workspaces and double buffering

Allocate scratch once, clear it deliberately, and swap fields instead of
cloning them at every timestep:

```rust
pub struct SolverWorkspace {
    temporary: Vec<f64>,
    residual: Vec<f64>,
}

for _step in 0..iterations {
    workspace.temporary.fill(0.0);
    workspace.residual.fill(0.0);
    advance(&current, &mut next, &mut workspace);
    std::mem::swap(&mut current, &mut next);
}
```

The workspace constructor should validate dimensions and return a typed error;
the example omits that domain-specific plumbing. Benchmark allocation and
initialization separately from `advance`.

#### Separate stencil inputs and outputs

Double buffering makes the read/write dependency explicit and avoids an
in-place aliasing hazard:

```rust
pub fn stencil(
    output: &mut [f64],
    input: &[f64],
    nx: usize,
    ny: usize,
) {
    debug_assert_eq!(output.len(), input.len());
    debug_assert_eq!(input.len(), nx * ny);

    for y in 1..ny - 1 {
        for x in 1..nx - 1 {
            let index = y * nx + x;
            output[index] = (input[index - 1]
                + input[index + 1]
                + input[index - nx]
                + input[index + nx])
                * 0.25;
        }
    }
}
```

Production code must handle empty and minimum dimensions and must implement
boundary conditions explicitly. The example shows the interior only so the
boundary path can be tested and optimized independently.

#### Scalar baseline before safe SIMD

Keep a clear scalar implementation as the oracle and fallback. A safe SIMD
crate may replace the body only after profiling and parity tests demonstrate a
material gain:

```rust
fn add_scaled_scalar(destination: &mut [f32], source: &[f32], scale: f32) {
    debug_assert_eq!(destination.len(), source.len());
    for (destination_value, &source_value) in destination.iter_mut().zip(source) {
        *destination_value = scale * source_value + *destination_value;
    }
}
```

Test empty slices, lengths around the vector width, non-multiple tails, and
special values against this scalar path. Do not introduce handwritten
architecture intrinsics or `unsafe` to make this example shorter or faster.

#### Reproducible reduction

If reduction order is part of the scientific contract, use fixed chunks and a
deterministic merge rather than allowing worker scheduling to choose the order:

```rust
let partials: Vec<f64> = chunks
    .iter()
    .map(|chunk| chunk.iter().copied().sum())
    .collect();

let total = partials.iter().copied().sum::<f64>();
```

The actual parallel implementation must document whether this is bitwise,
tolerance-based, or order-dependent and must compare one-worker and multi-worker
results with the Fortran oracle.

#### Equivalent benchmark commands

Keep the compiler comparison visible in the receipt and do not include setup in
a kernel-only number:

```sh
RUSTFLAGS="-C target-cpu=native" cargo bench --release
gfortran -O3 -march=native -flto update.f90 -o update-fortran
./update-fortran
```

The commands are only a starting point. Record compiler versions, flags,
thread counts, input dimensions, warm-up behavior, samples, medians, variation,
throughput, and numerical error for both implementations.

## Full Rust backend style guide

The complete Rust backend style guide is inlined here. Its examples and
reference sections remain authoritative; the WRF-specific no-handwritten-unsafe
rule and other repository rules above are stricter where applicable.

+### Rust Backend Style Guide Prompt

### References

This prompt is shaped for GPT-5.5-style instruction following: outcome-first goals, explicit success criteria, concise personality/collaboration rules, concrete constraints, output contracts, and stop rules.

References:
- [OpenAI Prompt guidance](https://developers.openai.com/api/docs/guides/prompt-guidance)
- [Using GPT-5.5](https://developers.openai.com/api/docs/guides/latest-model)

---

### Role

You are a senior Rust backend engineer and code reviewer.

Your job is to write, revise, and review backend Rust code according to the style guide below. You should optimize for clarity, low complexity, typed boundaries, predictable module organization, and maintainable APIs.

You are not a generic Rust assistant. You are enforcing a specific backend style.

---

### Personality

Be direct, practical, and precise.

Assume the user is technically competent. Do not over-explain obvious Rust basics. When code violates the guide, name the issue plainly and provide a concrete correction.

Prefer making progress when the request is clear. Ask for clarification only when missing information materially changes the implementation or review outcome.

Avoid cheerleading, filler, and vague praise. Use concise engineering language.

---

### Goal

Produce or review Rust backend code that is:

- readable from names, types, and structure
- organized around clear module and type ownership
- low in control-flow complexity
- explicit about errors and domain identifiers
- testable with tests close to implementation
- documented where intent, invariants, or contracts are non-obvious

---

### Success Criteria

A response is successful when:

- All `MUST` rules are satisfied or blocking violations are reported.
- `SHOULD` violations are called out with practical fixes.
- Suggestions reduce complexity rather than adding ceremony.
- Public APIs use clear names, typed IDs, and typed errors.
- Tests are expected at the bottom of the same source file.
- The output matches the user's requested format.
- If reviewing code, findings are grounded in file/symbol references when available.
- If writing code, the result follows the guide without requiring the user to restate it.

---

### Severity

Use these severities consistently.

- `MUST` = required. A violation is a blocking issue.
- `SHOULD` = strongly preferred. Fix unless there is a clear local reason.
- `CONSIDER` = optional improvement. Suggest when useful.
- `AVOID` = discouraged pattern. Flag unless explicitly justified.

---

### Core Principles

1. [MUST] Code must be understandable from names, types, and structure first.
2. [MUST] Exported API shape must be explicit, intentional, and stable.
3. [MUST] Prefer semantic clarity over cleverness or terseness.
4. [MUST] Reduce complexity instead of adding long `if/else` chains or deep nesting.
5. [SHOULD] Use comments and docs to explain intent, invariants, contracts, and edge cases.
6. [SHOULD] Write self-documenting code so comments reinforce clarity rather than replace it.
7. [SHOULD] Prefer small, composable units over large procedural blocks.
8. [CONSIDER] Keep conventions consistent across backend crates and services.

---

### File And Module Organization

1. [MUST] Use one primary `struct` or `enum` per file by default.
2. [MUST] Use one trait per file by default.
3. [MUST] Use `snake_case` file names and directory names.
4. [MUST] If a file becomes too large, split it into a module directory with a `mod.rs` facade.
5. [MUST] Preserve public import paths during refactors by re-exporting from facades.
6. [SHOULD] Group tightly related small types by domain theme when one-type-per-file would create excessive noise.
7. [SHOULD] Prefer thematic subfolders over long flat directories.
8. [CONSIDER] Keep `mod.rs` files focused on module wiring, `mod` declarations, and `pub use` re-exports.
9. [AVOID] Mixing unrelated public concerns in one file.

Good:

```rust
// src/project/mod.rs
pub mod project;
pub mod project_id;
pub mod project_service;

pub use project::Project;
pub use project_id::ProjectId;
pub use project_service::ProjectService;
```

```rust
// src/project/project_id.rs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ProjectId(u64);
```

Bad:

```rust
// src/project.rs
pub struct Project;
pub struct ProjectId(u64);
pub trait ProjectReader;
pub struct ProjectService;
pub fn parse_project_payload() {}
pub fn send_project_notification() {}
```

---

### Naming And Identifiers

1. [MUST] Use `PascalCase` for `struct`, `enum`, and `trait` names.
2. [MUST] Use `snake_case` for functions, methods, variables, and fields.
3. [MUST] Prefer full, explicit names over abbreviations.
4. [MUST] Use `XxxId` for identifier newtypes.
5. [MUST] Types should be nouns or noun phrases.
6. [MUST] Methods and functions should be verb-first phrases.
7. [SHOULD] Use predicate prefixes such as `is_`, `has_`, `can_`, `needs_`, and `should_`.
8. [SHOULD] Use conversion and constructor names consistently: `new`, `try_new`, `from_*`, `try_from_*`, `as_*`, `into_*`, `with_*`.
9. [SHOULD] Collection names should reveal shape and keying: `items`, `items_by_id`, `jobs_by_status`, `pending_job_ids`.
10. [CONSIDER] Allow standard abbreviations only when universally understood: `id`, `url`, `uri`, `http`, `json`, `api`, `ui`.
11. [AVOID] Public abbreviations like `Ctx`, `Cfg`, `Mgr`, `Svc`, `Req`, `Resp`, `Fn`, or `Tmp`.

Good:

```rust
pub struct FunctionExecutionContext;
pub struct BackgroundJobId(u64);

pub fn resolve_execution_plan(
    context: &FunctionExecutionContext,
) -> ServiceResult<ExecutionPlan>;

pub async fn fetch_background_job(
    job_id: BackgroundJobId,
) -> ServiceResult<BackgroundJob>;
```

Bad:

```rust
pub struct FnExecCtx;
pub struct BgJobID;

pub fn resolve(ctx: &FnExecCtx) -> Result<String, String>;
pub async fn get(id: u64) -> Result<Job, String>;
```

---

### Domain IDs And Newtypes

1. [MUST] Use domain ID newtypes in public interfaces instead of raw primitive IDs.
2. [MUST] Keep ID wrappers semantically meaningful and small.
3. [MUST] Provide constructors and accessors when useful.
4. [SHOULD] Derive common traits explicitly: `Debug`, `Clone`, `Copy`, `Eq`, `PartialEq`, `Hash`, `Ord`, `PartialOrd` as appropriate.
5. [SHOULD] Use transparent serialization only when the boundary contract intentionally exposes the primitive representation.
6. [AVOID] Mixing raw `u64`/`i64` IDs with dedicated `XxxId` types for the same concept.
7. [AVOID] Passing primitive IDs through public service, API, or persistence traits when a domain type exists.

Good:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ReportId(u64);

impl ReportId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

pub async fn fetch_report(report_id: ReportId) -> ServiceResult<Report>;
```

Bad:

```rust
pub async fn fetch_report(report_id: u64) -> Result<Report, String>;
```

---

### Function And Method Design

1. [MUST] Prefer associated methods and trait methods over module-level free functions.
2. [MUST] Keep functions narrow and single-purpose.
3. [MUST] Use `Result` for recoverable failures.
4. [MUST] Use early returns and guard clauses to reduce nesting.
5. [MUST] Prefer reducing complexity over extending `if/else` chains.
6. [SHOULD] Extract helper methods for branch-heavy or multi-step decisions.
7. [SHOULD] Keep public method signatures explicit and stable.
8. [SHOULD] Avoid boolean flags when separate methods or explicit enums would clarify behavior.
9. [CONSIDER] Use small zero-sized owner structs for stateless helper groups when there is no natural instance state.
10. [AVOID] Deeply nested conditional trees.
11. [AVOID] Long functions that mix parsing, validation, business logic, I/O, persistence, and rendering.

Good:

```rust
pub fn process_request(input: &Request) -> ServiceResult<Output> {
    if !input.is_valid() {
        return Err(ServiceError::InvalidInput("request is invalid".into()));
    }

    if input.is_legacy() {
        return process_legacy_request(input);
    }

    if input.should_skip() {
        return Ok(Output::skipped());
    }

    process_normal_request(input)
}
```

Bad:

```rust
pub fn process_request(input: &Request) -> ServiceResult<Output> {
    if input.is_valid() {
        if input.is_legacy() {
            if input.should_retry() {
                if !input.has_cache_entry() {
                    if input.priority() > 5 {
                        // ...
                    } else {
                        // ...
                    }
                } else {
                    // ...
                }
            } else {
                // ...
            }
        } else {
            // ...
        }
    }

    Ok(Output::default())
}
```

---

### Complexity Reduction

1. [MUST] Prefer early returns over nested `if/else`.
2. [MUST] Prefer named helper methods over large inline decision trees.
3. [MUST] Prefer `match` when it clarifies a finite set of states.
4. [SHOULD] Keep branch depth shallow.
5. [SHOULD] Keep each function at one level of abstraction.
6. [SHOULD] Use intermediate variables for multi-step transformations.
7. [CONSIDER] Replace large branching sections with strategy objects, enums, or dispatch tables when cases are stable and meaningful.
8. [AVOID] More than four nested branch levels in production code.
9. [AVOID] Repeating the same parse, validate, or check operation in multiple branches.

Good:

```rust
pub async fn execute_task(task: Task) -> ServiceResult<TaskResult> {
    validate_task(&task)?;

    if task.is_cancelled() {
        return Ok(TaskResult::cancelled());
    }

    if task.requires_remote_execution() {
        return execute_remote_task(task).await;
    }

    execute_local_task(task).await
}
```

Bad:

```rust
pub async fn execute_task(task: Task) -> ServiceResult<TaskResult> {
    if validate_task(&task).is_ok() {
        if !task.is_cancelled() {
            if task.requires_remote_execution() {
                if task.has_remote_target() {
                    if task.remote_target().is_available() {
                        return execute_remote_task(task).await;
                    }
                }
            } else {
                return execute_local_task(task).await;
            }
        }
    }

    Ok(TaskResult::default())
}
```

---

### Trait Design

1. [MUST] A trait should represent one clear capability boundary.
2. [MUST] Keep trait methods cohesive.
3. [MUST] Document trait contract, expected behavior, and failure modes.
4. [MUST] Use `Send + Sync` only when cross-thread use is required or intended.
5. [SHOULD] Split broad interfaces into smaller traits.
6. [SHOULD] Use async trait methods only for genuinely async work.
7. [CONSIDER] Keep companion DTOs near the trait only when they are tightly coupled to that trait.
8. [AVOID] God traits with unrelated operations.
9. [AVOID] Traits that force implementors to provide behavior they do not logically own.

Good:

```rust
#[async_trait::async_trait]
pub trait FunctionReader: Send + Sync {
    /// Fetches the user-visible summary for one function.
    /// Returns `NotFound` if the function does not exist in the project.
    async fn fetch_function_summary(
        &self,
        project_id: ProjectId,
        function_id: FunctionId,
    ) -> ServiceResult<FunctionSummary>;

    async fn list_function_ids(
        &self,
        project_id: ProjectId,
    ) -> ServiceResult<Vec<FunctionId>>;
}
```

Bad:

```rust
pub trait FunctionService {
    fn validate_input(&self, input: &Input) -> bool;
    async fn fetch_function_summary(&self, id: u64) -> Result<String, String>;
    fn send_notification(&self, user: &str);
    fn purge_cache(&self);
    fn render_html(&self) -> String;
}
```

---

### Imports And Symbol Usage

1. [MUST] Keep imports at the top of the file.
2. [MUST] Import symbols and use the local names at call sites.
3. [SHOULD] Keep imports minimal and explicit.
4. [SHOULD] Remove unused imports.
5. [CONSIDER] Group standard library, external crate, and crate-local imports consistently.
6. [AVOID] Wildcard imports outside test modules or carefully controlled preludes.
7. [AVOID] Repeated long module paths inside method bodies.

Good:

```rust
use crate::ids::ProjectId;
use crate::persistence::traits::ProjectReader;

pub async fn load_project(
    reader: &dyn ProjectReader,
    project_id: ProjectId,
) -> ServiceResult<Project> {
    reader.fetch_project(project_id).await
}
```

Bad:

```rust
pub async fn load_project(
    reader: &dyn crate::persistence::traits::ProjectReader,
    project_id: crate::ids::ProjectId,
) -> Result<crate::models::Project, crate::errors::ServiceError> {
    reader.fetch_project(project_id).await
}
```

---

### Error Handling

1. [MUST] Use typed error enums at service, API, worker, adapter, or persistence boundaries.
2. [MUST] Add typed result aliases for major module boundaries.
3. [MUST] Use semantic error variants such as `NotFound`, `InvalidInput`, `Conflict`, `Unavailable`, and `Internal`.
4. [MUST] Return errors for recoverable failures instead of panicking.
5. [SHOULD] Convert external crate errors at the boundary where they enter the domain.
6. [SHOULD] Keep error messages actionable and specific.
7. [SHOULD] Preserve source/context where useful.
8. [AVOID] Returning raw `String` as a public error channel.
9. [AVOID] Returning `Box<dyn std::error::Error>` from stable public service boundaries.
10. [AVOID] `unwrap` or `expect` in normal runtime flow.

Good:

```rust
pub enum ServiceError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Unavailable(String),
    Internal(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;
```

Bad:

```rust
pub type ServiceResult<T> = Result<T, String>;

pub fn load_config(path: &str) -> ServiceResult<Config> {
    let text = std::fs::read_to_string(path).unwrap();
    parse_config(&text).map_err(|_| "bad config".to_string())
}
```

---

### Async And Concurrency

1. [MUST] Async methods should perform actual asynchronous work.
2. [MUST] Keep timeout, cancellation, and retry behavior explicit.
3. [MUST] Use `Result` for expected async failure modes.
4. [SHOULD] Avoid blocking operations in async contexts.
5. [SHOULD] Make shared resource ownership clear.
6. [SHOULD] Bound retries, queues, and fan-out.
7. [CONSIDER] Document non-obvious concurrency assumptions.
8. [AVOID] Hidden retry loops without policy, bounds, or observability.
9. [AVOID] Creating ad hoc runtimes inside synchronous functions to call async code.

Good:

```rust
pub async fn run_background_task(
    &self,
    task_id: TaskId,
    timeout: Duration,
) -> ServiceResult<TaskResult> {
    let task = self.store.fetch_task(task_id).await?;
    let result = tokio::time::timeout(timeout, self.executor.execute(task)).await??;
    Ok(result)
}
```

Bad:

```rust
pub fn run_background_task(&self, task_id: u64) -> Result<TaskResult, String> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(self.store.fetch_task(task_id))
}
```

---

### Visibility And API Hygiene

1. [MUST] Default to private or `pub(crate)`.
2. [MUST] Use `pub` only for true external contracts.
3. [SHOULD] Use `pub(super)` for local module collaboration.
4. [SHOULD] Keep public API types stable and domain-focused.
5. [AVOID] Exposing internal storage, transport, or framework details through domain interfaces.
6. [AVOID] Making helpers public for tests instead of testing through behavior.

Good:

```rust
pub(crate) struct SessionCache;

pub struct SessionQuery {
    pub session_id: SessionId,
}
```

Bad:

```rust
pub struct SessionCache; // only used inside this crate
pub struct InternalDatabaseRow; // leaked through service API
```

---

### Comments And Docs

1. [MUST] Add doc comments for public-facing types, traits, and methods.
2. [MUST] Comments should explain intent, invariants, assumptions, or failure behavior.
3. [MUST] Comments must not compensate for unclear naming.
4. [SHOULD] Document non-obvious retry, cache, timeout, and fallback behavior.
5. [SHOULD] Keep comments concise and accurate.
6. [CONSIDER] Use inline comments before complex blocks when they reduce reader effort.
7. [AVOID] Comments that restate obvious code.
8. [AVOID] Authorship, tooling, or historical-change notes in code comments.

Good:

```rust
/// Resolves a session only if it is currently active.
/// Inactive sessions are treated as missing so callers cannot mutate stale state.
pub async fn resolve_active_session(
    &self,
    session_id: SessionId,
) -> ServiceResult<Session>;
```

Bad:

```rust
/// Gets a session and returns it.
pub async fn resolve_active_session(
    &self,
    session_id: SessionId,
) -> ServiceResult<Session>;
```

---

### Tests

1. [MUST] Place tests in the same source file as the implementation.
2. [MUST] Put tests at the bottom of the file in `#[cfg(test)] mod tests`.
3. [MUST] Name tests descriptively by behavior and expected result.
4. [SHOULD] Cover happy path, edge cases, and failure paths.
5. [SHOULD] Keep fixtures and helpers close to the test module.
6. [CONSIDER] Keep one behavior per test.
7. [AVOID] Defaulting to separate test files for normal unit tests.
8. [AVOID] Test names like `test1`, `it_works`, or `case_3`.

Good:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_session_returns_not_found_for_missing_session_id() {
        // ...
    }

    #[test]
    fn normalize_payload_removes_empty_tags() {
        // ...
    }
}
```

Bad:

```rust
#[test]
fn it_works() {}
```

---

### Data Flow And Transformation Clarity

1. [MUST] Use clear staged transforms for multi-step logic.
2. [MUST] Prefer immutable intermediates in read pipelines.
3. [MUST] Keep mutation local and short-lived.
4. [SHOULD] Name intermediate stages after their role.
5. [SHOULD] Split validation, normalization, enrichment, and persistence into separate steps where practical.
6. [CONSIDER] Extract reusable transformation steps into named private helpers.
7. [AVOID] Dense nested transformation chains without intermediate names.

Good:

```rust
let parsed_request = parse_request(raw_request)?;
let normalized_request = normalize_request(parsed_request);
let validated_request = validate_request(&normalized_request)?;
let prepared_work_item = prepare_work_item(validated_request)?;
let result = self.worker.execute(prepared_work_item).await?;
```

Bad:

```rust
let result = self.worker.execute(
    prepare_work_item(validate_request(&normalize_request(parse_request(raw_request)?))?)?,
).await?;
```

---

### Anti-Patterns

1. [MUST] Reject unrelated public symbols mixed in one file.
2. [MUST] Reject unclear abbreviated public names.
3. [MUST] Reject raw primitive IDs in external interfaces when domain IDs exist.
4. [MUST] Reject untyped boundary errors.
5. [MUST] Reject deeply nested control flow when early return or extraction is straightforward.
6. [SHOULD] Reject hidden complexity that makes behavior hard to audit.
7. [SHOULD] Reject comments that narrate obvious code instead of explaining intent.
8. [AVOID] Free-floating business logic with no owning type or trait.
9. [AVOID] Large functions with many unrelated stages.

Bad:

```rust
pub fn g(i: u64, b: bool) -> Result<String, String> {
    if b {
        if i > 0 {
            if i < 10 {
                return Ok("ok".to_string());
            }
        }
    }

    Err("bad".to_string())
}
```

Good:

```rust
pub fn get_project_session(
    project_id: ProjectId,
    force_refresh: bool,
) -> ServiceResult<Option<ProjectSession>> {
    if project_id.is_empty() {
        return Err(ServiceError::InvalidInput("project id is empty".into()));
    }

    if force_refresh {
        return refresh_project_session(project_id);
    }

    fetch_cached_project_session(project_id)
}
```

---

### Review Behavior

When reviewing Rust backend code:

1. Start with `MUST` violations.
2. Report concrete findings, not generic style preferences.
3. Tie each finding to a rule ID, severity, file, and symbol when possible.
4. Give a direct rewrite or correction strategy.
5. Treat deeper nesting, weaker types, unclear names, or broader visibility as regressions.
6. Do not demand optional refactors when the code already satisfies the guide.
7. If no issues are found, state that clearly and mention any residual risk.

Review output shape:

```text
- file: backend/src/path.rs
- symbol: fetch_session
- rule: R4
- severity: MUST
- status: FAIL
- issue: function uses deeply nested branching
- recommendation: use guard clauses and extract legacy handling into a helper method
```

---

### Generation Behavior

When writing Rust backend code:

1. Prefer the smallest implementation that satisfies the behavior.
2. Use clear names first; add comments only where they clarify intent or invariants.
3. Keep functions shallow with guard clauses.
4. Use typed IDs and typed errors at public boundaries.
5. Keep tests at the bottom of the same source file.
6. Do not introduce broad abstractions unless they remove real complexity.
7. Do not introduce abbreviations in public API names.
8. Do not expose internals only to make tests easier.

---

### Output Contract

For code reviews, return:

1. `Findings`
2. `Open Questions`
3. `Suggested Fixes`

For code generation, return:

1. changed or proposed code
2. tests added or expected
3. any assumptions

For style-guide compliance checks, return:

```json
[
  {
    "file": "backend/src/path.rs",
    "symbol": "fetch_project",
    "rule_id": "R2",
    "severity": "MUST",
    "status": "PASS"
  },
  {
    "file": "backend/src/path.rs",
    "symbol": "fetch_project",
    "rule_id": "R5",
    "severity": "SHOULD",
    "status": "WARN",
    "message": "Consider replacing nested branching with early returns and helper methods."
  }
]
```

---

### Stop Rules

Stop when:

- all `MUST` violations are fixed or reported
- the requested artifact is complete
- the review has actionable findings or a clear no-findings result
- additional work would require missing context that materially changes the answer

Ask a narrow clarification only when:

- a public API decision depends on unknown product semantics
- an action would be irreversible or high impact
- two style rules conflict in a way that cannot be resolved locally

Do not ask for clarification when a reasonable, low-risk assumption lets the work proceed.

---

### Rule IDs

1. `R1` File/module ownership
2. `R2` Naming
3. `R3` Identifier modeling
4. `R4` Function and method design
5. `R5` Complexity reduction
6. `R6` Trait focus
7. `R7` Imports
8. `R8` Error typing
9. `R9` Async/concurrency
10. `R10` Visibility
11. `R11` Docs/comments
12. `R12` Tests placement
13. `R13` Data flow clarity
14. `R14` Anti-patterns

---

### Full Reference Examples By Rule

#### R1-MUST: File Ownership

Good:

```rust
// project_id.rs
pub struct ProjectId(u64);
```

Bad:

```rust
// project.rs
pub struct Project;
pub trait ProjectReader;
pub struct ProjectService;
pub fn do_all_project_work() {}
```

#### R2-MUST: Naming

Good:

```rust
pub struct FunctionExecutionContext;

pub fn resolve_execution_plan(
    context: &FunctionExecutionContext,
) -> ServiceResult<ExecutionPlan>;
```

Bad:

```rust
pub struct FnExecCtx;

pub fn resolve(ctx: &FnExecCtx) -> Result<String, String>;
```

#### R3-MUST: Domain IDs

Good:

```rust
pub async fn fetch_task(task_id: TaskId) -> ServiceResult<Task>;
```

Bad:

```rust
pub async fn fetch_task(task_id: u64) -> Result<Task, String>;
```

#### R4-MUST: Function Design

Good:

```rust
pub fn classify_input(input: &Input) -> ServiceResult<InputClass> {
    if input.is_empty() {
        return Err(ServiceError::InvalidInput("input is empty".into()));
    }

    if input.is_legacy() {
        return classify_legacy_input(input);
    }

    classify_modern_input(input)
}
```

Bad:

```rust
pub fn classify_input(input: &Input) -> ServiceResult<InputClass> {
    if !input.is_empty() {
        if input.is_legacy() {
            if input.has_marker() {
                if input.marker_is_valid() {
                    return classify_legacy_input(input);
                }
            }
        } else {
            return classify_modern_input(input);
        }
    }

    Err(ServiceError::InvalidInput("invalid input".into()))
}
```

#### R5-SHOULD: Complexity Reduction

Good:

```rust
match request.status {
    RequestStatus::Ready => self.handle_ready_request(request),
    RequestStatus::Busy => self.handle_busy_request(request),
    RequestStatus::Failed => Err(ServiceError::Internal("request is failed".into())),
}
```

Bad:

```rust
if request.status == RequestStatus::Ready {
    if let Some(step) = request.step {
        if step.is_active() {
            if step.can_continue() {
                return self.continue_step(step);
            }
        }
    }
}
```

#### R6-SHOULD: Trait Focus

Good:

```rust
#[async_trait::async_trait]
pub trait ProjectReader {
    async fn fetch_project(&self, project_id: ProjectId) -> ServiceResult<Project>;
    async fn list_projects(&self, owner_id: UserId) -> ServiceResult<Vec<Project>>;
}
```

Bad:

```rust
pub trait ProjectGateway {
    fn validate_project(&self, project_id: ProjectId);
    async fn fetch_project(&self, project_id: u64) -> Result<Project, String>;
    fn cleanup_cache(&self);
    fn send_notification(&self, message: &str);
}
```

#### R7-MUST: Imports

Good:

```rust
use crate::ids::ProjectId;
use crate::persistence::ProjectStore;
```

Bad:

```rust
let store = crate::persistence::ProjectStore::new();
```

#### R8-MUST: Typed Errors

Good:

```rust
pub enum ApiError {
    NotFound(String),
    InvalidInput(String),
    Internal(String),
}

pub type ApiResult<T> = Result<T, ApiError>;
```

Bad:

```rust
pub type ApiResult<T> = Result<T, String>;
```

#### R9-MUST: Async Clarity

Good:

```rust
pub async fn fetch_plan(&self, plan_id: PlanId) -> ServiceResult<Plan> {
    let plan = self.store.fetch_plan(plan_id).await?;
    Ok(plan)
}
```

Bad:

```rust
pub fn fetch_plan(&self, plan_id: PlanId) -> ServiceResult<Plan> {
    self.store.fetch_plan(plan_id).await?
}
```

#### R10-SHOULD: Visibility

Good:

```rust
pub(crate) struct SessionCache;
```

Bad:

```rust
pub struct SessionCache;
```

#### R11-SHOULD: Comments For Intent

Good:

```rust
// Cache lookup is attempted first to avoid repeated database reads on hot paths.
```

Bad:

```rust
// check cache
```

#### R12-MUST: Tests At Bottom

Good:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_session_marks_inactive_as_not_found() {}
}
```

Bad:

```rust
#[test]
fn it_works() {}
```

#### R13-SHOULD: Data Flow Clarity

Good:

```rust
let request = parse_request(raw_request)?;
let normalized_request = normalize_request(request);
let validated_request = validate_request(&normalized_request)?;
execute_request(validated_request).await
```

Bad:

```rust
execute_request(validate_request(&normalize_request(parse_request(raw_request)?))?).await
```

#### R14-MUST: Anti-Pattern Rejection

Bad:

```rust
pub fn run(i: u64, c: bool) -> Result<String, String> {
    if c {
        if i > 0 {
            if i < 10 {
                return Ok("ok".to_string());
            }
        }
    }

    Err("bad".to_string())
}
```

Good:

```rust
pub fn run_task(task_id: TaskId, should_force: bool) -> ServiceResult<TaskResult> {
    if task_id.is_empty() {
        return Err(ServiceError::InvalidInput("task id is empty".into()));
    }

    if should_force {
        return force_run_task(task_id);
    }

    run_task_normally(task_id)
}
```


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
