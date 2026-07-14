# First WRF physics scheme selection

Date: 2026-07-13
Upstream: WRF v4.7.1, commit `f52c197ed39d12e087d02c50f412d90d418f6186`

## Decision

The first translated physical parameterization is Kessler warm-rain
microphysics from `phys/module_mp_kessler.F`.

Kessler is a complete WRF scheme rather than a helper routine. It advances
water vapor, cloud water, rain water, potential temperature, and two
precipitation diagnostics. Its columns are horizontally independent, giving a
real CPU-parallel boundary without requiring the unported physics driver.

## Candidate inventory

| Candidate | Pinned size | Dependency/state shape | Decision |
|---|---:|---|---|
| Kessler microphysics | 244 lines | One self-contained subroutine; eight 3-D fields, two 2-D precipitation fields, scalar constants | Selected |
| Slab land surface | 556 lines | Surface and soil state, category tables, driver-specific surface contracts, initialization routine | Deferred until surface-state ownership exists |
| MRF boundary layer | 1,407 lines | Column diffusion, tridiagonal solve, many tendencies and diagnostics | Deferred; scientifically useful after thermodynamic column conventions are established |
| WSM3 microphysics | 1,578 lines | Broader phase-change state and shared constants | Deferred until the warm-rain field/workspace boundary is proven |
| BMJ cumulus | 2,194 lines | Deep driver coupling and large column state | Deferred until convective driver contracts are mapped |
| Held-Suarez radiation/damping | Already translated | Idealized forcing already lives in `wrf-dynamics` | Excluded because issue #6 requires the first new physics scheme |

Line counts describe the pinned source files, not implementation difficulty.
The selection prioritizes a dependency-closed scientific column with observable
state changes and a direct compiler oracle.

## State ownership

The scheme borrows all model fields for one update:

- mutable potential temperature `t`;
- mutable water-vapor, cloud-water, and rain-water mixing ratios `qv`, `qc`,
  and `qr`;
- immutable dry-air density `rho`;
- immutable Exner function `pii`;
- immutable mass-level height `z`;
- immutable W-level layer thickness `dz8w`; and
- mutable accumulated and step precipitation `RAINNC` and `RAINNCV`.

`CpuKesslerMicrophysicsWorkspace` owns scratch, not model state. Its full-size
production field corresponds to WRF's local `prod` array. It also owns one
vertical terminal-velocity buffer per persistent worker. The workspace is
created during setup and reused across timesteps.

## Dependency closure

`module_mp_kessler.F` contains no `USE` dependencies. Every physical constant
needed by the subroutine is passed as a scalar except six coefficients declared
inside the routine. The Rust parameter object validates the passed constants
once and provides a constructor for the exact WRF v4.7.1 defaults.

The public kernel capability uses backend-owned field and workspace associated
types. CPU code may use host slices and Rayon internally; neither appears in
the cross-backend scientific method signature. A future GPU backend can retain
the same region, parameters, fields, and operation while supplying device
storage and device scratch.

## Parity policy

The initial policy is raw IEEE-754 equality for every finite output. The
fixture contains exponential, square-root, and fractional-power operations,
so exact equality is stronger than a broad tolerance. The oracle compiles the
pinned module directly and compares all 660 mutable values, including excluded
halos and both precipitation fields.

No tolerance is currently accepted. If a future platform's conforming math
library changes transcendental results, the first-divergence report must be
reviewed before introducing a narrowly documented ULP policy. A loose
end-state tolerance is not an automatic fallback.

## Parallel execution

WRF stores fields in `(i,k,j)` order. The Rust CPU field has the same logical
linear order: west-east points are contiguous, followed by bottom-top, then
south-north. Kessler's outer `j` rows are independent, so the CPU backend
schedules complete south-north rows across its persistent work-stealing pool.

Sedimentation remains sequential within each column because vertical fallout
at one level depends on its neighbor. Warm-rain conversion preserves WRF's
`k`-then-`i` order inside each row. Disjoint mutable row slices eliminate locks
for model fields. A short uncontended lock selects the preallocated terminal
velocity buffer belonging to the current worker.

## Explicit exclusions

This slice does not yet include:

- the WRF microphysics driver or scheme dispatch;
- moisture-species Registry indexing;
- coupling into an ARW Runge-Kutta step;
- ice, snow, graupel, or aerosol processes;
- distributed halo exchange around the scheme call; or
- an end-to-end precipitation trajectory.

Those are integration gates, not hidden claims of scheme completeness.
