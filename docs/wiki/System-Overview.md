# System overview

## What WRF is

The Weather Research and Forecasting model is not one solver. It is a modeling
system whose executable is assembled from a dynamical core, a registry-driven
state model, domain decomposition and halo exchange, physical parameterization
suites, initialization programs, nesting, and scientific I/O. WRF v4.7.1 also
contains related systems such as data assimilation and chemistry. A successful
Rust port must therefore reproduce contracts between subsystems, not merely
translate isolated equations.

The current target is the Advanced Research WRF (ARW) core. Whole-model parity
remains zero until Rust can initialize and advance an upstream case; completed
utility slices are foundations, not evidence of forecast parity.

## Workspace layers

`wrf-rs` currently has seven crates:

- `wrf-time` owns civil-calendar conversion, exact model timestamps, intervals,
  and clocks. It corresponds to WRF's bundled ESMF-derived time manager.
- `wrf-compute` owns backend-neutral field shape/storage contracts and the
  default persistent CPU executor. It contains no weather equations.
- `wrf-registry` owns typed build-time Registry parsing and selected generated
  artifacts. It contains no live domain fields.
- `wrf-domain` owns typed domain, patch, memory, and tile bounds plus
  transport-neutral halo plans and the deterministic local reference executor.
- `wrf-domain-mpi` owns the MPI adapter. Keeping it separate prevents MPI
  communicator types from entering scientific kernel interfaces.
- `wrf-dynamics` owns translated ARW numerical capabilities. Its first kernel
  families cover positive-definite correction, Held-Suarez damping, and
  selected `rk_step_prep` calculations: column-mass staggering, momentum
  coupling, dry-air omega diagnosis, and moisture momentum coefficients.
- `wrf-physics` owns translated physical parameterizations. Its first scheme is
  Kessler warm-rain microphysics with backend-native reusable workspace.
- `wrf-io` owns typed WRF NetCDF schema, borrowed dataset validation, classic
  64-bit-offset output, NetCDF-3/4 input, and bounded exact restart comparison.

This split follows ownership. Time can be tested without a grid, compute can be
tested without atmospheric formulas, Registry generation can be tested without
a live domain, and domain semantics can be tested without a numerical kernel.
Future I/O and model-integration crates should be added at similarly real
boundaries rather than as one crate per Fortran file.

## Intended model-step flow

The following is an architectural target, not yet an implemented executable:

1. read registry-derived configuration and a WRF initialization dataset;
2. allocate backend-native fields once for a domain and its halos;
3. initialize a model clock and domain decomposition;
4. dispatch dynamics and physics capabilities for each timestep;
5. exchange halos and synchronize only at required dependency boundaries;
6. emit history/restart fields with WRF-compatible dimensions and metadata;
7. compare checkpoint fields and diagnostics with the pinned WRF run.

## Porting philosophy

WRF defines the scientific and observable contract. Fortran implementation
details do not. Rust code may replace allocatable scratch arrays with in-place
disjoint mutation, implicit integer conventions with typed indices, and manual
thread management with a trusted work-stealing runtime. Such changes are
preferred when they improve safety, readability, or performance and the parity
suite proves the same required output.

`unsafe` is forbidden in every current crate. If a future dependency contains
internally audited unsafe code, it must be mature, justified, and hidden behind
a safe API; local unsafe code requires an explicit architectural decision and
is not the default escape hatch for performance.
