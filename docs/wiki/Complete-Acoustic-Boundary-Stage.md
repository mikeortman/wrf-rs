# Complete acoustic boundary stage

`AcousticBoundaryStageKernels` is the local-memory translation of WRF v4.7.1's
`solve_em.F` acoustic window. It composes the verified trajectory, specified
boundary updates, nonhydrostatic geopotential update, zero-gradient W handling,
and physical boundary-zone assignment behind one failure-atomic call.

## Exact call sequence

Before the small-step loop the stage performs:

1. `small_step_prep`;
2. initialization `calc_p_rho`;
3. `calc_coef_w`; and
4. physical boundaries for U/V tendencies, PH, inverse density, pressure,
   previous theta, saved theta, previous/current column mass, and divergence
   damping column mass, in upstream order.

Each of three configured acoustic substeps then performs:

1. `advance_uv`, followed by specified/nested U and V updates;
2. `advance_mu_t`, followed by specified/nested theta, MU, and MUTS updates;
3. `advance_w`;
4. `sumflux`;
5. `spec_bdyupdate_ph`, then specified zero-gradient W or nested W tendency
   update;
6. advancing `calc_p_rho`; and
7. physical boundaries for PH, inverse density, pressure, MUTS, MU, and MUDF.

Periodic domains skip specified updates but still execute their periodic
physical copies. Specified domains use `zero_grad_bdy` for W; nested domains
use `spec_bdyupdate`.

## Controls and ownership

The stage accepts the same borrowed time levels, saved state, diagnostics,
workspace, inputs, and vertical coefficients as the local trajectory. U and V
tendencies are mutable borrows because WRF applies physical boundary copies to
them before `advance_uv`; all other true inputs remain immutable. No model field
is cloned, and the caller owns the vertical-solve workspace.

`AcousticBoundaryStageControls` derives the trajectory's lateral policies from
one `PhysicalBoundaryConditions` value. This prevents callers from supplying a
specified physical configuration alongside contradictory global numerical
regions. Typed region roles distinguish U faces, V faces, mass half levels,
horizontal mass, and full levels.

## Whole-call preflight

The CPU backend validates the complete trajectory first, then every physical
field, specified destination/tendency pair, geopotential coefficient, and W
boundary source. Polar filtering and hydrostatic mode are rejected because this
slice owns the nonhydrostatic PH/W path but not WRF's `pxft` polar operation.

No mutation occurs until every late-stage dependency has passed validation.
Tests inject a wrong theta region after otherwise valid trajectory inputs and
prove every mutable field is unchanged. A polar configuration is similarly
rejected atomically.

## Parity evidence

The direct driver extracts all seven acoustic numerical routines plus
`spec_bdyupdate`, `spec_bdyupdate_ph`, `zero_grad_bdy`, and both physical
boundary routines from the pinned WRF source. The Fortran and Rust fixtures use
role-distinct, coordinate-dependent finite values for every supported mutable
field, tendency, pressure/geopotential input, moisture coefficient, mass, map
factor, and vertical coefficient. This makes field swaps and argument-order
mistakes observable rather than allowing equal constants to hide them.

After preparation, pressure initialization, coefficient construction, every
boundary insertion, every numerical substage, and all three substep closures,
the driver emits complete raw-bit storage for every field that operation
mutates. That is 149 affected-field snapshots and 184,725 intermediate values
per case, rather than sparse point probes. It then emits complete final storage
for U, V, W, theta, PH, inverse density, pressure, MU, MUTS, MUDF, and all three
accumulated fluxes.

The six direct cases are full-tile periodic, specified, and nested domains; a
specified west-edge partial tile (`i=4..6`, `j=5..8`); a specified interior tile
(`i=5..8`, `j=5..8`) on which physical and specified boundary branches are
inactive; and a full specified tile with injected negative zero, positive
infinity, quiet NaN, negative infinity, and maximum finite `f32` values. Across
the six cases, Rust matches 1,108,350 intermediate values and 98,550 final-field
values. Finite values, infinities, and signed zero require exact bits; NaNs match
by class. Independent tests prove one-worker and four-worker complete-state
equality for all six cases.

## Current boundary

This is a single-rank, nonpolar, nonhydrostatic stage. Halo exchange,
communication overlap, polar filtering, boundary-file interpolation, and
Registry-generated state binding belong to the next driver layer. The direct
[performance record](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/acoustic-boundary-stage-2026-07-14.md)
measures the integrated stage rather than summing component estimates.
