# Compute backend architecture

Status: accepted for the CPU-first port; GPU implementation is intentionally
deferred.

## Decisions

1. `CpuBackend::try_new()` is the standard runtime path and uses all host
   parallelism reported by `std::thread::available_parallelism`. There is no
   single-threaded Cargo feature and no opt-in parallel feature.
2. CPU results are the parity reference. A translated kernel first passes its
   upstream WRF fixtures on CPU before any device implementation is accepted.
3. Fields are backend-owned. `FieldStorage` exposes logical shape, while
   `CpuField` alone exposes host slices. A future `GpuField` may remain resident
   in device memory.
4. `ComputeBackend` owns allocation only. Each numerical subsystem will define
   a narrow capability trait for its kernels, such as a dynamics tendency or a
   microphysics column update. This avoids a god trait as the port grows.
5. Arbitrary Rust closures are a CPU scheduling facility, not the cross-backend
   kernel API. GPU implementations will dispatch native kernels behind the
   same subsystem capability traits.
6. Grid shape and memory order remain explicit. A backend change may not alter
   logical dimensions, staggering, halo semantics, precision, or parity policy.

## CPU kernel pattern

A CPU kernel captures immutable input slices and gives each persistent
work-stealing worker a disjoint mutable output chunk. The scheduler provides the
global linear range, so kernels retain stable indexing without locks or unsafe
aliasing. The pool is created once; timesteps do not create OS threads.

## GPU readiness gate

GPU work begins only after a kernel has:

- a typed CPU API and error boundary;
- upstream-derived input and output fixtures;
- documented floating-point comparison rules;
- explicit field layout and halo requirements;
- no dependency on host pointers in its capability trait.

This design permits GPU support without making today’s CPU code pay for a fake
device abstraction or weakening the reference implementation.
