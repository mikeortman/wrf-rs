# Specified-boundary relaxation

## Purpose

A limited-area WRF domain cannot evolve its lateral edge independently. A
parent domain or external boundary file supplies the desired state and its time
tendency. The outer specified zone is imposed directly; an adjacent relaxation
zone blends the interior solution toward that supplied state while damping
small horizontal discontinuities.

WRF v4.7.1 implements this operation in `share/module_bc.F` through
`relax_bdytend`, `relax_bdytend_tile`, and their shared
`relax_bdytend_core`. The two wrappers differ only in the allocation represented
by the input field: a complete memory field or a tile-sized field extended by
one horizontal neighbor.

## Mismatch field

For a point at distance `d` from a boundary, define the boundary mismatch

```text
L0 = boundary(d) + dtbc * boundary_tendency(d) - field
```

The boundary state is therefore advanced to the requested boundary time before
it is compared with the model field. Four additional mismatches use the two
tangential neighbors and the immediately outer and inner normal neighbors:

```text
L1, L2 = tangential predecessor and successor
L3     = outward neighbor at distance d - 1
L4     = inward neighbor at distance d + 1
```

WRF adds the following source-ordered expression to the existing tendency:

```text
tendency += fcx(d) * L0
          - gcx(d) * (L1 + L2 + L3 + L4 - 4 * L0)
```

`fcx` controls direct relaxation toward the external state. `gcx` applies a
five-point Laplacian of the mismatch, smoothing abrupt horizontal variation.
The Rust kernel retains WRF's single-precision evaluation order because
algebraically equivalent regrouping can change final IEEE-754 bits.

## Zone geometry

The `spec_zone` outer points are not modified here; direct specified-boundary
assignment owns them. Relaxation begins at distance `spec_zone` and stops just
inside distance `relax_zone - 1`. Every relaxed distance requires both `d - 1`
and `d + 1` boundary records. Consequently, a nonempty relaxation band requires
a positive specified-zone width and at least `relax_zone + 1` stored boundary
points.

South and north are traversed first. At nonperiodic corners, a row at distance
`d` excludes `d` columns from both ends. West and east then exclude one more
row, which prevents normal domains from updating a corner twice. If X is
periodic, west/east forcing is disabled and south/north rows span the full
active X range.

The five field locations share this horizontal geometry but differ in their
upper stagger:

| Location | WRF selector | Extra point |
|---|---|---|
| Mass half level | default | none |
| West–east face | `u` | upper X |
| South–north face | `v` | upper Y |
| Horizontal mass | `m` | exactly one vertical level |
| Full level | `h` | upper vertical level |

## Rust ownership and storage

`SpecifiedBoundaryRelaxationKernels` is a narrow backend capability. Its field
associated type remains native to the backend, leaving room for a future GPU
implementation. The CPU implementation borrows:

- one immutable model field;
- one mutable tendency field;
- four boundary-state fields;
- four boundary-tendency fields; and
- two small coefficient slices.

`SpecifiedBoundaryRelaxationField` pairs the immutable allocation with the
model-coordinate ranges it represents. Full storage uses the whole allocation.
A tile field uses only its tile-plus-one-neighbor ranges, reproducing
`relax_bdytend_tile` without a patch-sized copy or unsafe pointer offset.

All shapes, zone relationships, coefficient lengths, and stencil-neighbor
coverage are checked before the first write. Invalid input therefore leaves the
entire tendency unchanged. Execution assigns disjoint south–north planes to the
persistent Rayon pool and retains south, north, west, east source order within
each plane. There is no numerical scratch allocation or field clone.

## Parity evidence

The differential harness extracts the exact three pinned Fortran routines. Ten
cases compare all 5,500 stored tendency values by raw bits across mass, U, V,
full-level, and horizontal-mass fields; full and halo-extended tile storage;
periodic X; opposite partial tiles; inactive and empty bands; signed zero,
subnormal, maximum finite, and infinite inputs.

Additional Rust tests exhaustively compare the half-open range planner with a
literal source-loop membership model, prove one/four-worker determinism, and
verify failure atomicity for every boundary role and validation category.

On a matched 238,080-point workload, four-worker and default sixteen-worker
Rust are within 15% of optimized serial Fortran. The standard multithreaded path
is therefore accepted without explicit SIMD until integrated profiling shows a
material need.

## Integration boundary

This page documents the isolated numerical kernel. The next gate inserts
boundary-file tendency assignment, this relaxation step, specified updates,
and halo exchange around the already verified local acoustic trajectory. Only
that coupled fixture can prove correct ordering across tiles and substeps.
