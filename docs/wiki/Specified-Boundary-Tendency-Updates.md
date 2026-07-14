# Specified-boundary tendency updates

## Role in ARW

Nested and specified WRF domains receive externally supplied boundary values.
During time integration, `spec_bdyupdate` adds the corresponding tendency only
inside the outer specified zone:

```text
field(i,k,j) = field(i,k,j) + dt * field_tend(i,k,j)
```

WRF invokes this operation for horizontal momentum, potential temperature,
column mass, scalar tendencies, and vertical momentum. The arithmetic is
simple; the important behavior is the C-grid staggering and corner geometry.

## Boundary geometry

South and north zones are trapezoids. At distance `d` from either boundary,
the nonperiodic update excludes `d` points from each west/east corner. West and
east side zones then exclude one additional row beyond that distance. The four
loops therefore cover the specified perimeter without double-updating ordinary
corners.

With periodic X, west/east updates are suppressed and the south/north zones
become rectangular. This matches the pinned routine's `periodic_x` branch.

## Field locations

`SpecifiedBoundaryFieldLocation` replaces WRF's character selector:

| Rust location | WRF selector behavior | Upper point included |
|---|---|---|
| `MassHalfLevel` | default scalar/thermodynamic field | none |
| `WestEastFace` | `u` | west–east |
| `SouthNorthFace` | `v` | south–north |
| `HorizontalMass` | `m` | one supplied horizontal level |
| `FullLevel` | `h` | vertical |

`SpecifiedBoundaryUpdateRegion` validates storage, physical mass/half-level
domains, and a tile that may contain the single upper stagger point. It then
clips execution to the selected field location. Validation occurs before the
first store, so shape failures leave the destination unchanged.

## Parallel and memory behavior

The CPU backend assigns complete south–north planes to the persistent worker
pool. Within each plane it derives at most four direct boundary ranges in WRF
source order. It does not scan inactive interior points, allocate numerical
scratch, clone fields, or require unsafe code.

On the matched 256 × 256 × 40 workload, one-worker Rust is 1.49× faster than
optimized serial Fortran, four-worker Rust is 3.88× faster, and the default
16-worker path is 1.61× faster. Explicit SIMD is deferred because ordinary
safe Rust already clears the performance gate.

## Parity evidence

The oracle extracts the exact WRF v4.7.1 routine body. Eight cases compare all
1,728 stored values by raw IEEE-754 bits across mass, U, V, horizontal-mass,
and full-level fields; periodic X; partial south/west tiles; and an inactive
interior tile. Added tests cover zero-width zones, shape failure atomicity, and
one/four-worker complete-output determinism.

The next integration gate ports the remaining boundary-finalization routines
and inserts boundary and halo operations around the local acoustic trajectory.
