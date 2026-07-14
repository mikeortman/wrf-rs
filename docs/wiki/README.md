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
- [WRF Registry](WRF-Registry.md) — the Registry DSL, preprocessing and typed
  Rust model, generated-state pipeline, supported subset, and parity evidence.
- [WRF NetCDF and restarts](WRF-NetCDF-and-Restarts.md) — staggered schema,
  typed zero-copy I/O, exact restart comparison, and current limitations.
- [Domain decomposition and halo exchange](Domain-Decomposition-and-Halo-Exchange.md)
  — process partitions, patch/memory/tile bounds, periodic endpoints, corner
  propagation, local execution, and MPI transport.
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
- [Momentum coupling](Momentum-Coupling.md) — mass/map-factor equations,
  stagger-specific clipping, typed field ownership, safe vector-friendly rows,
  and exact-bit evidence.
- [Kessler warm-rain microphysics](Kessler-Microphysics.md) — sedimentation,
  cloud conversion, saturation adjustment, reusable workspace, parallel rows,
  and exact pinned-WRF evidence.
- [Development infrastructure](Development-Infrastructure.md) — source pinning,
  scripts, verification gates, and documentation policy.
- [Positive-definite performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/positive-definite-2026-07-13.md)
  — release throughput, scaling, generated-code findings, and caveats.
- [Held-Suarez performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/held-suarez-2026-07-13.md)
  — CPU scaling and the matched optimized-Fortran comparison.
- [Column-mass staggering performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/column-mass-staggering-2026-07-13.md)
  — matched geometry, CPU scaling, allocation evidence, and the rejected SIMD
  screen.
- [Momentum-coupling performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/momentum-coupling-2026-07-13.md)
  — matched C-grid workload, bounds-check optimization, scaling, and allocation
  evidence.
- [Kessler microphysics performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/kessler-microphysics-2026-07-13.md)
  — matched optimized-Fortran timing, CPU scaling, and scratch/allocation
  accounting.
- [NetCDF restart I/O performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/netcdf-restart-2026-07-14.md)
  — control-plane and bulk throughput, peak memory, buffering decision, and
  remaining dependency gap.
- [Rust/Fortran performance ledger](https://github.com/mikeortman/wrf-rs/blob/main/PERFORMANCE_PARITY.md) — matched
  workload policy and cumulative comparison table.
- [Rust module structure](https://github.com/mikeortman/wrf-rs/blob/main/docs/architecture/module_structure.md) — family-owned
  source hierarchy and stable crate facades.

## Maintenance rule

Each completed port slice updates its algorithm page, parity evidence,
performance notes, crate-level Rust documentation, the root `README.md`,
`TEST_COVERAGE.md`, and `CURRENT_STATE.md`. Findings in the Fortran source also
go into `UPSTREAM_FINDINGS.md` with a confidence label and reproduction.
