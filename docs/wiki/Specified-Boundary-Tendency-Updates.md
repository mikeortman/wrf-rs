# Specified-boundary updates

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

## Mass-normalized geopotential

Nested nonhydrostatic domains use `spec_bdyupdate_ph` after each acoustic flux
accumulation. It applies the same boundary geometry but accounts for the
changing dry column mass. For level `k`, define

```text
mu_old = muts - dt * mu_tend
old_mass = c1(k) * mu_old + c2(k)
new_mass = c1(k) * muts + c2(k)
```

WRF then evaluates, in single-precision source order,

```text
ph = ph * old_mass / new_mass
   + dt * ph_tend / new_mass
   + ph_save * (old_mass / new_mass - 1)
```

`SpecifiedBoundaryGeopotentialInputs` separates the three-dimensional saved
geopotential and tendency from the two-dimensional column-mass fields and
one-dimensional vertical coefficients. The capability validates every shape
and coefficient length before mutation. It computes `mu_old` as a scalar at
the consuming point instead of materializing WRF's tile-sized automatic array.

Nine direct cases compare all 1,944 stored values across every supported field
location, periodic X, partial and inactive tiles, and zero-denominator IEEE
behavior. Finite values, signed zeros, and infinities match by raw bits; NaNs
match by class because their sign and payload are compiler-dependent.

On the matched 256 × 256 × 41 workload, serial Rust is 2.01× faster than
optimized serial Fortran, four-worker Rust is 5.63× faster, and the default
16-worker path is 3.25× faster. The kernel uses no numerical scratch or field
clones, so explicit SIMD is not justified for this slice.

## Zero-gradient vertical-momentum boundaries

After applying specified tendencies to nonhydrostatic vertical momentum, WRF
calls `zero_grad_bdy`. Each boundary destination receives the value at the
nearest independent interior row or column. South and north copies clamp the
source column to the interior core on nonperiodic domains; periodic X keeps the
same column and suppresses west/east copies. West and east copies similarly
clamp the source row.

The vertical loop has a subtle source contract: it begins at the tile's lower
vertical bound but ignores the tile's upper bound, continuing through the
physical domain top. W fields include the extra upper full level. The Rust
kernel encodes this behavior explicitly and validates that the specified zone
leaves at least one independent interior source on every active axis before
the first mutation.

Seven direct cases compare all 3,584 stored values by raw bits across W,
default, U, and V locations; periodic X; a partial south/west tile with a
clipped vertical start; and an inactive interior tile. Added tests cover a
zero-width no-op, missing-interior failure atomicity, shape failure atomicity,
and one/four-worker determinism.

On the matched 256 × 256 × 41 workload, one-worker Rust is 6.8% slower than
optimized serial Fortran and four-worker Rust is 1.21× faster. The kernel has
no numerical scratch or field clones. This is close enough to stop: explicit
SIMD or special worker selection would add complexity without evidence of an
end-to-end benefit.
