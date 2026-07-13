# Execution and storage

## Logical dimensions and physical order

`GridShape` names the west-east, south-north, and bottom-top extents and checks
their product for overflow. CPU fields are one contiguous allocation. For the
positive-definite sheet, each west-east line is contiguous, matching Fortran
`f(nx, ny)`, where the first index varies fastest.

The distinction between logical shape and physical storage matters. A future
GPU backend may tile or pad memory, but it may not silently change staggering,
halo ownership, precision, or field meaning. Layout transformations must occur
at explicit boundaries and be covered by parity tests.

## CPU execution

`CpuBackend` creates one Rayon thread pool and shares it through `Arc`. The
normal constructor uses the parallelism reported by the host; explicit worker
counts exist for resource limits and deterministic tests. A timestep does not
spawn operating-system threads.

Two safe scheduling forms currently exist:

- adaptive chunks for ordinary elementwise output work; and
- exact contiguous blocks for columns, lines, or profiles that must not be
  divided internally.

Rayon work stealing balances uneven line costs. Closures receive disjoint
mutable output slices, so Rust proves absence of write aliasing without locks or
local unsafe code. Immutable input slices may be shared across workers.

## Capability traits

`ComputeBackend` only allocates fields. Numerical crates define focused traits,
such as `PositiveDefiniteKernels`, with an associated native field type. The
CPU implementation can use host slices; a future GPU implementation can launch
a device kernel on resident buffers.

This avoids two common abstraction failures: a single ever-growing “backend”
trait containing every weather scheme, and a closure API that claims to be GPU
portable even though arbitrary Rust closures cannot execute on a device.

## SIMD and reductions

SIMD is an inner-kernel concern, while Rayon supplies outer parallelism. Runtime
instruction detection should occur once per kernel dispatch, then vector code
runs inside worker-owned blocks. `pulp` is the leading candidate for runtime
dispatch; `wide` is useful for controlled targets. Nightly `std::simd` is not a
stable baseline.

Reduction order is part of floating-point behavior. The positive-definite
kernel keeps minimum and sum reductions scalar and ordered because a vector
tree reduction can change rounding. Its pointwise translate/scale passes are
better SIMD candidates once profiling demonstrates value.
