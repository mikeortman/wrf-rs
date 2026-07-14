# Complete local acoustic trajectory

The local acoustic trajectory is the first Rust capability that executes WRF's
entire nonhydrostatic small-step numerical chain in production order. It is the
bridge between individually verified kernels and a future distributed ARW
time-step driver.

## Sequence

`AcousticTrajectoryKernels::advance_acoustic_trajectory` performs:

1. `small_step_prep` once;
2. the initialization form of `calc_p_rho` once;
3. `calc_coef_w` once;
4. `advance_uv` for each acoustic substep;
5. `advance_mu_t` for that substep;
6. `advance_w` for that substep;
7. `sumflux` for that substep; and
8. the advancing form of `calc_p_rho` to close that substep.

This order is taken directly from WRF v4.7.1 `dyn_em/solve_em.F`. Pressure is
therefore available to the next horizontal-momentum update, while the vertical
solve coefficients remain fixed for the complete local sequence.

## Ownership model

The public API groups borrowed data by lifetime and scientific role:

- `AcousticTrajectoryTimeLevels` owns no storage; it borrows the mutable
  previous/current prognostic fields;
- `AcousticTrajectorySavedState` borrows the large-step reference fields
  produced by preparation;
- `AcousticTrajectoryDiagnostics` borrows pressure, mass, solver-factor, and
  time-averaged flux outputs;
- `AcousticTrajectoryWorkspace` borrows the one reusable volume field required
  by the implicit vertical solve;
- `AcousticTrajectoryInputs` contains immutable mass, pressure, tendency,
  moisture, map-factor, and terrain descriptors;
- `AcousticTrajectoryCoefficients` borrows ten one-dimensional vertical arrays;
  and
- `AcousticTrajectoryRegions` reuses the seven already validated kernel regions.

The CPU implementation constructs short-lived stage descriptors. It does not
clone model fields or allocate numerical scratch inside the sequence.

## Failure atomicity

Every stage is structurally validated before preparation mutates the first
field. This matters because a shape error in the final flux accumulator would
otherwise leave the model halfway through an acoustic step. The test suite
injects that late failure and compares every mutable field before and after the
call.

Numerical runtime failures are limited to the existing typed kernel errors and
worker failures. Communication, physical boundary updates, polar filtering,
and nested forcing remain explicit caller responsibilities; the local
capability does not pretend they are computation-local operations.

## Interpolation coefficients

WRF names its full-level interpolation weights `fnm` and `fnp`. Their
lower/upper naming in legacy call surfaces is easy to reverse because `fnm`
multiplies the upper full-level value while `fnp` multiplies the lower value.
The trajectory API calls these `upper_full_level_weight` and
`lower_full_level_weight`, then maps them deliberately into each kernel's
existing argument order.

## Parity evidence

`scripts/run-acoustic-trajectory-oracle.sh` extracts all seven exact routine
bodies from the pinned WRF source, compiles them together, and executes three
nonhydrostatic acoustic substeps. Rust compares 2,196 final values for current
U, V, W, theta, geopotential, column mass, pressure, inverse-density
perturbation, and all three time-averaged mass fluxes. Every value matches its
Fortran IEEE-754 bits.

Separate tests prove one-worker and four-worker complete-state bit equality.
The coupled oracle complements rather than replaces each routine's broader
branch, boundary, exceptional-value, and inactive-storage corpus.

## Current boundary

This capability covers one local, nonpolar, nonperiodic, nonnested,
nonhydrostatic tile. The next integration layer must insert WRF's halo exchange,
physical-boundary, polar-filter, and specified/nested update points between the
local numerical stages, then bind the fields to Registry-generated model state.
