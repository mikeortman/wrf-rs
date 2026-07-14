# Acoustic horizontal momentum

WRF's `advance_uv` advances the two horizontally staggered momentum components
during each small acoustic step. U lives on west-east cell faces; V lives on
south-north faces. This geometry determines the pressure-gradient neighbor and
the physical-boundary points each equation may update.

## Algorithm

Each component first receives its large-step tendency. The kernel then
evaluates WRF's split pressure gradient: perturbation-geopotential difference,
inverse density times perturbation-pressure difference, and perturbation
inverse density times base-pressure difference, scaled by hybrid mass and map
factors.

Nonhydrostatic mode adds pressure interpolated from half to full levels. The
lower boundary uses `cf1:cf3`, interior levels use `fnm/fnp`, and the upper full
level is zero for a nonrigid top or three-level extrapolation for a rigid lid.
Hydrostatic mode omits this fourth term. A centered column-mass difference adds
divergence damping. Rust preserves the two visible rounding stages: tendency
update, then pressure/damping update.

## Bounds and boundaries

Typed domain objects distinguish mass domains, upper C-grid points, storage,
and tiles. Open edges suppress pressure updates. Symmetric edges suppress both
tendency and pressure updates. Relaxation zones clip the physical interior;
periodic X restores the full west-east tile. Polar edges suppress V pressure
updates and force active V values to positive zero.

## Execution, memory, and evidence

The CPU backend gives each worker a disjoint south-north plane. Level-major,
contiguous west-east loops require neither locks nor unsafe code. Rust allocates
none of Fortran's `dpn`, `dpxy`, or `mudf_xy` scratch arrays; values are
computed and consumed locally. The storage-generic capability remains suitable
for a future native GPU kernel.

The direct oracle extracts pinned WRF v4.7.1 `advance_uv` and matches every U/V
bit in a rigid-lid nonhydrostatic case. Added tests cover governing modes, top
boundaries, polar rows, relaxation/periodic ranges, validation atomicity, and
worker determinism.

WRF's partial nonhydrostatic vertical-tile path can read uninitialized `dpn` at
`k_start`; see WRF-046 in `UPSTREAM_FINDINGS.md`. Rust evaluates the intended
interpolation directly and is deterministic for partial tiles.
