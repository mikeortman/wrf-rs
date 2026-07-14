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

The oracle extracts the exact WRF v4.7.1 routine body. Nine cases compare all
1,944 stored values by raw IEEE-754 bits across mass, U, V, horizontal-mass,
and full-level fields; periodic X; partial south/west and shortened upper
vertical tiles; and an inactive interior tile. Added tests cover zero-width
zones, shape failure atomicity, and one/four-worker complete-output
determinism.

The five-point relaxation stage is now documented separately in
[Specified-boundary relaxation](Specified-Boundary-Relaxation.md). The next
integration gate inserts both tendency stages plus boundary and halo operations
around the local acoustic trajectory.

## Boundary-file tendency assignment

Before applying and relaxing specified boundaries, `spec_bdytend` copies the
time tendency supplied by the boundary file into the prognostic tendency field.
It accepts both boundary state and boundary tendency arrays, but the pinned
routine reads only the four tendency arrays. Rust therefore models only the
data dependency that affects output through `SpecifiedBoundaryTendencies`.

The assignment reuses the exact trapezoidal geometry above. Each destination
receives the side value at its line coordinate, vertical level, and distance
from the boundary. Periodic X suppresses west/east assignment and makes the
south/north strips rectangular.

The vertical contract depends on field location. Mass-half, U, and V fields
start at the tile's lower bound but ignore its upper bound, continuing to the
physical half-level top. Horizontal mass and full-level fields respect the
supplied upper tile bound; full-level fields may include the extra top point.
The shared typed region now captures this distinction for both
`spec_bdytend` and `spec_bdyupdate`.

Twelve direct cases compare all 2,592 stored values by raw bits across every
field location, periodic X, opposite partial tiles, shortened vertical tiles,
inactive and zero-width cases, signed zero, infinities, and a subnormal. Shape
and width failures leave the entire mutable tendency unchanged, and one/four
worker executions are bitwise identical.

On the matched 256 × 256 × 40 workload, serial Rust is 1.19× faster and
four-worker Rust is 2.91× faster than optimized serial Fortran. The default
16-worker path is 3.2% slower, which is operationally equal for this thin copy.
The kernel allocates no numerical scratch and clones no fields, so custom
scheduling and explicit SIMD are not justified without an integrated profile.

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

## Flow-dependent scalar boundaries

Moisture species without supplied lateral conditions, TKE, tracers, and some
scalars use `flow_dep_bdy`. The field is unstaggered, while coupled U and V
velocities classify each contacted boundary point:

| Boundary | Outflow condition | Outflow value | Inflow value |
|---|---|---|---|
| south | `v(i,k,j) < 0` | nearest interior scalar row | `+0.0` |
| north | `v(i,k,j+1) > 0` | nearest interior scalar row | `+0.0` |
| west | `u(i,k,j) < 0` | nearest interior scalar column | `+0.0` |
| east | `u(i+1,k,j) > 0` | nearest interior scalar column | `+0.0` |

The upper-side tests require one stored U or V neighbor beyond the last scalar
point. `SpecifiedBoundaryFlowRegion` fixes the field location to unstaggered
mass half levels, and the capability validates the scalar and both velocity
shapes, upper neighbors, and independent interior core before mutation.

As with `zero_grad_bdy`, the vertical loop begins at the tile's lower bound but
ignores its upper bound, continuing through the half-level domain top. Six
direct cases compare all 3,072 stored values by raw bits, including periodic X,
both partial-boundary orientations, inactive storage, NaN, signed-zero, and
infinite velocity signs. Added tests cover shape, neighbor, and core failures
atomically plus one/four-worker determinism.

On the matched 256 × 256 × 40 workload, serial Rust is 1.5% faster than
optimized serial Fortran and four-worker Rust is 1.24× faster. No numerical
scratch or field clones are used. The default 16-worker pool is overhead-bound
for this thin operation; special scheduling and SIMD wait for an integrated
scalar-advancement profile.

### Inflow policies

WRF provides three copies of the flow-dependent traversal whose only material
difference is the action taken at an inflow point. Rust represents that choice
with `SpecifiedBoundaryInflowPolicy` and shares the geometry and velocity-sign
logic:

| Rust policy | WRF routine | Inflow action |
|---|---|---|
| `Zero` | `flow_dep_bdy` | write positive zero |
| `Constant(value)` | `flow_dep_bdy_qnn` | write `ccn_conc` exactly |
| `Preserve` | `flow_dep_bdy_fixed_inflow` | retain the destination |

The constant-policy oracle includes finite values, negative zero, and positive
infinity. Six direct cases compare all 3,072 stored values by raw bits across
constant and preserve policies, periodic X, partial tiles, and the ignored
upper tile bound. One- and four-worker tests also compare complete output.

On the matched 256 × 256 × 40 workload, four-worker Rust is 1.09× faster than
optimized serial Fortran for constant inflow and 2.6% faster for preserve.
Serial Rust is 15.0% and 22.4% slower, respectively. Since the normal
four-worker path is already competitive, specialization and explicit SIMD are
deferred in favor of one readable traversal.

## Final state reconstruction

At the end of the acoustic work, `spec_bdy_final` forces prognostic state back
to the time-interpolated boundary-file value. This prevents round-off drift
that would accumulate if WRF relied only on applied tendencies. For boundary
value `b`, boundary tendency `b_t`, and interpolation interval `dtbc`, it first
evaluates

```text
b_interpolated = b + dtbc * b_t
```

The field location then selects normalization:

| Field location | WRF selector | Final value |
|---|---|---|
| horizontal mass | `m` | `b_interpolated` |
| scalar half level | `t` or default | `b_interpolated / (c1 * mu + c2)` |
| full level | `h` | `b_interpolated / (c1 * mu + c2)` |
| U, V, or W momentum | `u`, `v`, `w` | `map_factor * b_interpolated / (c1 * mu + c2)` |

The Rust region replaces the selector with six typed locations and reuses the
verified trapezoidal corner geometry. It also records a non-obvious source
contract: every location starts at the tile's lower vertical bound but ignores
the upper bound, continuing to the applicable physical half- or full-level
top.

Eleven direct cases compare all 5,184 stored values by raw bits across every
location, periodic X, partial and inactive tiles, signed zero, and infinities.
All eight oriented boundary value/tendency arrays, both normalization fields,
both coefficient vectors, and invalid widths fail before mutation.

On the matched 256 × 256 × 41 vertical-momentum workload, four-worker Rust is
1.50× faster and default 16-worker Rust is 1.36× faster than optimized serial
Fortran. The kernel has no numerical scratch or field clones; further SIMD or
scheduler specialization waits for an integrated boundary-driver profile.
