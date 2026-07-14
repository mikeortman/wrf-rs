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
| `positive_definite_sheet` | Pinned Fortran routine compiled directly; 9 focused exact-bit cases plus 24 seeded randomized cases | epsilon boundary, signed zero, finite magnitude extremes, explicit NaN/infinity policy, invalid dimensions/totals, one-vs-four-worker determinism, statistical throughput and warmed allocation budgets | operational field distributions; trajectory integration |
| `positive_definite_slab` | Pinned Fortran routine compiled directly; focused sentinel fixture plus 16 seeded randomized cases | typed half-open region validation, varied non-one memory origins, broader domain/tile clipping, signed zero, finite magnitude extremes, explicit NaN/infinity policy, untouched storage, throughput/scaling and allocation budgets | randomized worker-count determinism; operational field distributions |
| `held_suarez_damp` | Pinned Fortran module compiled directly; 16 focused selections plus 12 seeded randomized complete-field cases | varied non-one memory origins and clipped domains, signed zero, finite magnitude extremes, explicit NaN/infinity policy, staggered-neighbor/range validation, shape mismatch before mutation, one-vs-four-worker determinism, scalar/SIMD raw-bit parity for lengths 1–257, release scaling and allocation baselines | x86-64 SIMD benchmark; complete Held-Suarez trajectory |
| `calc_mu_staggered` | Exact routine body extracted from the pinned large module; 240 focused values plus 16 seeded randomized complete-field cases cover all 16 cross-axis boundary combinations | varied non-one memory origins, clipped staggered rectangles, signed zero, finite magnitude extremes, explicit NaN/infinity policy, validation before mutation, one-vs-four-worker determinism, untouched halos, matched benchmark/scaling/allocation evidence, rejected parity-correct SIMD screen | periodic `calc_mu_uv` variants; full ARW integration |

## Fixture policy

Golden files are generated from a named pinned upstream build, never from the
Rust implementation. Floating-point fixtures store raw IEEE-754 bits where
exact parity is expected. A tolerance is allowed only when the responsible
algorithm page documents why exact ordering cannot or should not be retained,
and the test reports absolute, relative, and ULP error separately.

The seeded ARW corpora contain 68 cases and 39,588 complete output values.
Finite values and infinities compare by raw bits. NaN outputs compare by class,
because payload and sign propagation are not portable model contracts. Every
failure reports the seed, field, and first divergent linear index. The oracle
regenerates and byte-compares the committed inputs before compiling Fortran, so
generator drift cannot silently rewrite the test population.
