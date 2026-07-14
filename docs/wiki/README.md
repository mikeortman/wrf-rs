# wrf-rs technical wiki

This wiki is the durable conceptual map for both contributors and future agent
sessions. It explains what the code does, why the architecture exists, how it
maps to WRF, and where numerical assumptions are verified. Pages describe the
implemented system; proposed work is explicitly labeled.

## Start here

- [System overview](System-Overview.md) — major WRF subsystems and the Rust
  workspace map.
- [Execution and storage](Execution-and-Storage.md) — memory order,
  multithreading, capability traits, and the future GPU boundary.
- [Parity and testing](Parity-and-Testing.md) — what “same output” means and how
  Fortran oracles prevent self-confirming tests.
- [Randomized differential testing](Randomized-Differential-Testing.md) —
  shared raw-bit inputs, deterministic seeds, exceptional-value policy, and
  first-divergence diagnosis.
- [Timekeeping](Timekeeping.md) — exact rational model time and Gregorian
  calendar behavior.
- [Positive-definite correction](Positive-Definite-Correction.md) — derivation,
  branch semantics, layout, and performance characteristics of the first
  numerical kernel.
- [Held-Suarez momentum damping](Held-Suarez-Damping.md) — pressure-dependent
  Rayleigh friction, staggered pressure geometry, parallel execution, and
  exact-bit evidence.
- [Column-mass staggering](Column-Mass-Staggering.md) — C-grid interpolation,
  physical-boundary copy rules, domain/tile/storage contracts, operation order,
  and exact-bit evidence.
- [Development infrastructure](Development-Infrastructure.md) — source pinning,
  scripts, verification gates, and documentation policy.
- [Positive-definite performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/positive-definite-2026-07-13.md)
  — release throughput, scaling, generated-code findings, and caveats.
- [Held-Suarez performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/held-suarez-2026-07-13.md)
  — CPU scaling and the matched optimized-Fortran comparison.
- [Column-mass staggering performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/column-mass-staggering-2026-07-13.md)
  — matched geometry, CPU scaling, allocation evidence, and the rejected SIMD
  screen.
- [Rust/Fortran performance ledger](https://github.com/mikeortman/wrf-rs/blob/main/PERFORMANCE_PARITY.md) — matched
  workload policy and cumulative comparison table.
- [Rust module structure](https://github.com/mikeortman/wrf-rs/blob/main/docs/architecture/module_structure.md) — family-owned
  source hierarchy and stable crate facades.

## Maintenance rule

Each completed port slice updates its algorithm page, parity evidence,
performance notes, crate-level Rust documentation, the root `README.md`,
`TEST_COVERAGE.md`, and `CURRENT_STATE.md`. Findings in the Fortran source also
go into `UPSTREAM_FINDINGS.md` with a confidence label and reproduction.
