# CPU SIMD strategy

Status: accepted direction; adoption is kernel-by-kernel after scalar parity.

## Stable-Rust baseline

The CPU reference remains stable Rust. `std::simd` is still a nightly feature,
so it is not a production dependency. The two leading stable crates have
different roles:

- `pulp` provides safe runtime instruction-set detection and dispatch plus
  manual vectorization. It is the preferred first candidate for distributed
  binaries that must select an implementation on the machine where they run.
- `wide` provides clear fixed-width vector types and strong cross-platform
  coverage, but selects x86 instruction support at build time. It remains a
  useful candidate for controlled deployment targets and kernels where its API
  materially improves clarity.

No SIMD crate is added globally before a real WRF kernel needs it. That avoids
locking every numerical crate to an abstraction selected from synthetic code.

## Kernel workflow

Each vectorized kernel must have three layers:

1. a scalar implementation used as the readable parity oracle;
2. an automatically vectorizable contiguous loop where LLVM code generation is
   inspected in release mode;
3. an explicit SIMD implementation only when profiling shows the kernel is hot
   and manual vectorization is faster on representative x86-64 and AArch64
   machines.

Runtime SIMD selection happens once per kernel dispatch, never once per grid
point. SIMD runs inside the disjoint chunks already assigned by the persistent
CPU pool; it does not create another concurrency layer.

## Numerical constraints

- Vector lanes process the same scalar formula and precision as upstream WRF.
- Tail elements use the same implementation semantics as full vectors.
- Reductions document their order. A faster tree reduction is not parity-safe
  merely because its real-number equation is equivalent.
- Fused multiply-add is enabled only when WRF's compiler configuration and the
  comparison policy permit the resulting rounding change.
- Transcendental approximations require per-function error bounds and direct
  comparison with the configured WRF reference build.

## Acceptance evidence

A SIMD path is accepted only with upstream-derived parity fixtures, scalar vs.
SIMD differential tests, release-mode benchmarks above and below vector-width
boundaries, and generated-code inspection for every supported architecture.

## First generated-code finding

The 2026-07-13 AArch64 release inspection of the positive-definite kernels found
scalar floating-point instructions in the translation, reduction, and scaling
loops; LLVM did not emit packed NEON arithmetic. See
`docs/performance/positive-definite-2026-07-13.md`. This justifies prototyping
`pulp` for the independent pointwise passes while retaining scalar ordered
reductions. It does not by itself prove that manual SIMD will improve total
kernel throughput.

The subsequent `pulp` 0.22.3 prototype achieved exact bit parity but regressed
the representative one- and four-worker cases by roughly 1–4%. It improved
only the 16-worker measurements. The path was therefore removed; see the
baseline report for the full table. Explicit SIMD remains an evidence-based
per-kernel choice, not a workspace-wide requirement.
