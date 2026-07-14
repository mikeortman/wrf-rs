# Test coverage and gap inventory

This is the test-planning ledger. The root `README.md` tracks implementation
scope; this file tracks confidence within that scope.

## Required layers

Every numerical slice should accumulate five kinds of evidence:

1. **Upstream regression parity** — port every applicable named upstream test.
2. **Differential oracle coverage** — compile the pinned Fortran routine and
   compare identical fixtures with Rust.
3. **Adversarial coverage** — add branch boundaries, malformed shapes, signed
   zero, extrema, and scientifically meaningful invariants absent upstream.
4. **Concurrency determinism** — compare one worker with multiple workers and
   exercise scheduling failures.
5. **Operational coverage** — compare full fields, restarts, I/O metadata, and
   trajectories in representative WRF cases.

## Current matrix

| Surface | Upstream parity | Added gap tests | Remaining gaps |
|---|---|---|---|
| `wrf-time` | 93/93 active `Test1.F90` cases; `ESMF_` and `WRFU_` golden outputs exact | year zero, negative year, rational normalization, invalid components | randomized long-clock sequences; leap-second policy when a caller requires it |
| CPU chunk scheduler | Not an upstream surface | disjoint writes, synchronization-confirmed concurrent workers in debug/release, typed kernel error, worker panic | nested-pool behavior; large NUMA systems; cancellation semantics |
| CPU exact-block scheduler | Not an upstream surface | exact boundaries, invalid shapes, worker panic | empty-output contract; scaling and allocation measurements |
| `positive_definite_sheet` | Pinned Fortran routine compiled directly; 9 exact-bit cases | epsilon boundary, signed zero, invalid dimensions/totals, one-vs-four-worker determinism, statistical throughput and warmed allocation budgets | NaN/infinity policy; randomized differential corpus |
| `positive_definite_slab` | Pinned Fortran routine compiled directly; exact-bit active-region and sentinel fixture | typed half-open region validation, non-one memory-origin translation, domain/tile clipping, untouched halo and stagger points, throughput/scaling and allocation budgets | signed zero and exceptional floats; randomized corpus; broader domain/memory/tile combinations; worker determinism |
| `held_suarez_damp` | Pinned Fortran module compiled directly; 16 exact-bit active/boundary selections | non-one memory origins, staggered-neighbor and range validation, shape mismatch before mutation, one-vs-four-worker determinism, scalar/SIMD raw-bit parity for lengths 1–257, release scaling and allocation baselines | randomized differential corpus; NaN/infinity policy; broader clipped domains; x86-64 SIMD benchmark; complete Held-Suarez trajectory |
| `calc_mu_staggered` | Exact routine body extracted from the pinned large module and compiled; 240 raw-bit output/sentinel values cover interior, lower, upper, and both-boundary paths independently on both axes | typed separation and validation of memory shape, mass domain, and active momentum tile; clipped staggered rectangles; shape failure before mutation; one-vs-four-worker determinism at all physical boundaries; untouched halos | periodic `calc_mu_uv` variants; randomized corpus; exceptional-float policy; matched Fortran benchmark and allocation budget |

## Fixture policy

Golden files are generated from a named pinned upstream build, never from the
Rust implementation. Floating-point fixtures store raw IEEE-754 bits where
exact parity is expected. A tolerance is allowed only when the responsible
algorithm page documents why exact ordering cannot or should not be retained,
and the test reports absolute, relative, and ULP error separately.
