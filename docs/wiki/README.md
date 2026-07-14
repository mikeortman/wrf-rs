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
- [Dry-air omega diagnosis](Omega-Diagnosis.md) — continuity integration,
  complete-column contract, map-factor fluxes, scratch-free parallel rows, and
  exact-bit evidence.
- [Moisture momentum coefficients](Moisture-Momentum-Coefficients.md) — moist
  mass correction, species ordering, C-grid averaging, stagger clipping,
  scratch-free parallel rows, and exact-bit evidence.
- [Full inverse density](Full-Inverse-Density.md) — base-state reconstruction,
  mass-grid clipping, typed field roles, contiguous parallel rows, and exact-bit
  evidence.
- [Pressure-point geopotential](Pressure-Point-Geopotential.md) — vertical
  full-level averaging, source operation order, upper-neighbor validation,
  parallel rows, and exact-bit evidence.
- [Runge-Kutta diagnostic preparation](Runge-Kutta-Preparation.md) — the
  seven-stage dependency chain, failure-atomic ownership boundary, coupled
  exact-bit oracle, and integrated performance.
- [Dry Runge-Kutta tendency assembly](Dry-Tendency-Assembly.md) — persistent
  physics/boundary tendencies, map-factor coupling, staggered ranges, safe
  paired-output scheduling, and exact-bit evidence.
- [Acoustic small-step preparation](Acoustic-Step-Preparation.md) — time-level
  switching, mass-coupled perturbations, C-grid/full-level contracts, and exact
  first/later-substep evidence.
- [Acoustic pressure and inverse-density diagnosis](Acoustic-Pressure-Diagnosis.md)
  — nonhydrostatic/hydrostatic equations, pressure-history damping, vertical
  geopotential recurrence, and exact mode/phase evidence.
- [Vertical acoustic solve coefficients](Vertical-Acoustic-Coefficients.md) —
  tridiagonal factorization, complete-column and top-boundary contracts,
  parallel XZY traversal, and exact coefficient evidence.
- [Acoustic horizontal momentum](Acoustic-Horizontal-Momentum.md) — split U/V
  pressure gradients, C-grid boundaries, scratch-free parallel execution, and
  exact pinned-source evidence.
- [Acoustic mass, omega, and potential temperature](Acoustic-Mass-Omega-and-Theta.md)
  — continuity integration, vertical mass flux, theta transport, complete-column
  contracts, output reuse, and exact pinned-source evidence.
- [Implicit acoustic vertical momentum and geopotential](Implicit-Acoustic-Vertical-Momentum-and-Geopotential.md)
  — geopotential transport, terrain reconstruction, tridiagonal vertical solve,
  upper damping, reusable workspace, and exact pinned-source evidence.
- [Acoustic flux accumulation](Acoustic-Flux-Accumulation.md) — staggered
  running sums, final linear recoupling, first-step clearing, parallel storage,
  and exact three-substep evidence.
- [Complete local acoustic trajectory](Complete-Local-Acoustic-Trajectory.md) —
  the seven-kernel execution order, failure-atomic ownership boundary,
  interpolation roles, and exact coupled three-substep evidence.
- [Specified-boundary updates](Specified-Boundary-Tendency-Updates.md) — C-grid
  field locations, trapezoidal corners, periodic-X behavior, mass-normalized
  geopotential, direct parallel ranges, and exact pinned-source evidence.
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
- [Omega-diagnosis performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/omega-diagnosis-2026-07-13.md)
  — matched complete-column workload, row-layout correction, CPU scaling, and
  allocation evidence.
- [Moisture-coefficient performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/moisture-coefficients-2026-07-13.md)
  — matched six-species workload, CPU scaling, output-as-scratch design, and
  allocation evidence.
- [Full inverse-density performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/inverse-density-2026-07-14.md)
  — matched mass-grid workload, CPU scaling, allocation evidence, and the SIMD
  stopping decision.
- [Pressure-point geopotential performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/pressure-point-geopotential-2026-07-14.md)
  — matched vertical-average workload, CPU scaling, allocation evidence, and
  the source-order SIMD stopping decision.
- [Runge-Kutta preparation performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/runge-kutta-preparation-2026-07-14.md)
  — matched seven-diagnostic workload, CPU scaling, allocation evidence, and
  the cross-stage optimization stopping decision.
- [Dry-tendency assembly performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/dry-tendency-assembly-2026-07-14.md)
  — matched first-substep workload, CPU scaling, paired-output allocation
  evidence, and the SIMD stopping decision.
- [Acoustic-step preparation performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/acoustic-step-preparation-2026-07-14.md)
  — matched first-substep workload, CPU scaling, allocation evidence, and the
  complexity stopping decision.
- [Acoustic-pressure performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/acoustic-pressure-2026-07-14.md)
  — matched governing modes, hydrostatic layout correction, CPU scaling,
  allocations, and the SIMD stopping decision.
- [Vertical-acoustic coefficient performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/vertical-acoustic-coefficients-2026-07-14.md)
  — matched tridiagonal construction, layout correction, CPU scaling,
  allocations, and the deferred SIMD opportunity.
- [Implicit acoustic vertical-momentum performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/acoustic-vertical-momentum-2026-07-14.md)
  — matched complete-column solve, CPU scaling, reusable workspace accounting,
  and the SIMD stopping decision.
- [Acoustic flux-accumulation performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/acoustic-flux-accumulation-2026-07-14.md)
  — matched three-substep workload, staggered output scaling, allocation
  evidence, and the SIMD stopping decision.
- [Complete local acoustic-trajectory performance estimate](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/acoustic-trajectory-2026-07-14.md)
  — exact stage counts, equivalent optimization levels, aggregate timing, and
  the direct-integrated-benchmark boundary.
- [Specified-boundary update performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/specified-boundary-update-2026-07-14.md)
  — matched boundary geometry, direct-range correction, CPU scaling,
  allocation evidence, and the SIMD stopping decision.
- [Specified-boundary geopotential performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/specified-boundary-geopotential-2026-07-14.md)
  — matched full-level geometry, mass normalization, CPU scaling, allocation
  evidence, and the SIMD stopping decision.
- [Zero-gradient specified-boundary performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/zero-gradient-boundary-2026-07-14.md)
  — matched nearest-interior copies, CPU scaling, allocation evidence, and the
  readability-first tuning stop.
- [Flow-dependent specified-boundary performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/flow-dependent-boundary-2026-07-14.md)
  — matched inflow/outflow classification, CPU scaling, allocation evidence,
  and the integrated-profile tuning boundary.
- [Flow-dependent inflow-policy performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/flow-dependent-inflow-policies-2026-07-14.md)
  — matched constant and preserve policies, CPU scaling, allocation evidence,
  and the readability-first tuning stop.
- [Specified-boundary finalization performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/specified-boundary-finalization-2026-07-14.md)
  — matched mass/map-factor reconstruction, CPU scaling, allocation evidence,
  and the multithreaded tuning stop.
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
