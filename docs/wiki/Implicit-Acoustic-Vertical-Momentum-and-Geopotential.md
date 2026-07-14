# Implicit acoustic vertical momentum and geopotential

WRF's `advance_w` completes the nonhydrostatic part of each small acoustic
step. It advances coupled vertical momentum, updates perturbation geopotential,
and maintains a normalized time-averaged thermodynamic term used by vertical
buoyancy. Unlike the horizontal momentum equations, the vertical pressure
gradient is solved implicitly because vertically propagating acoustic waves
would otherwise impose a restrictive timestep.

## Place in the acoustic step

The preceding `calc_coef_w` routine constructs the lower diagonal, reciprocal
eliminated diagonal, and upper elimination factor for each complete column.
`advance_w` then performs these stages:

1. time-center and normalize the thermodynamic state;
2. build the geopotential right-hand side from its large-step tendency,
   vertical momentum, and vertical transport;
3. impose the terrain-following lower vertical-velocity condition;
4. add vertical pressure-gradient and buoyancy tendencies;
5. apply forward substitution and backward substitution;
6. optionally damp upper-level vertical motion; and
7. recover the new perturbation geopotential.

The Rust capability keeps these stages separate in `right_hand_side.rs`,
`momentum.rs`, and `geopotential.rs`. The public trait describes the scientific
operation while the CPU backend owns its storage traversal. A future GPU
backend can implement the same capability with device-native fields and one
device RHS workspace.

## Geopotential right-hand side

The initial right-hand side combines the geopotential tendency with the
off-centered vertical-momentum term. Vertical geopotential transport has two
WRF discretizations:

- `StaggeredGeopotentialGradient` first forms the staggered vertical gradient
  and then multiplies by omega. This corresponds to `phi_adv_z = 2`.
- `StaggeredTransportProduct` first destaggers omega, multiplies by the local
  gradient, and then interpolates the product.

Both are public enum states rather than an undocumented integer. Source
operation order is retained, including the multiplication order that affects
single-precision rounding. The rigid-lid policy sets the upper RHS to zero
before the implicit solve, exactly as WRF does.

## Terrain-following lower boundary

At the surface, geometric vertical velocity follows the terrain slope:

\[
w_s = m_y\,u_y\,\partial_y h + m_x\,u_x\,\partial_x h.
\]

Centered north/south and east/west terrain differences are multiplied by the
three vertically weighted velocity levels nearest the surface. The typed
region therefore requires west, east, south, and north terrain neighbors and
at least three mass levels. Specified or nested domains exclude one lateral
edge point; periodic west-east treatment restores only the west-east range.

## Implicit vertical solve

Interior and top vertical momentum first receive the large-step tendency,
pressure-gradient correction, inverse-density buoyancy contribution, and
time-centered column-mass term. The precomputed factors then solve the
tridiagonal system without another factorization:

\[
\hat w_k = (w_k - a_k\hat w_{k-1})\alpha_k,
\qquad
w_k = \hat w_k - \gamma_k w_{k+1}.
\]

The first recurrence moves upward through the complete column. The second
moves downward, excluding the fixed terrain level and already solved top
level. These dependencies prevent parallelism within one column, but columns
and south-north planes remain independent. The standard CPU backend schedules
complete planes on its persistent work-stealing pool.

## Upper damping

WRF `damp_opt = 3` applies a squared-sine weight within `zdamp` meters of the
model top. Rust represents this as `AcousticVerticalDamping::UpperLayer` with
an explicit coefficient and depth. `Disabled` has no branch-specific scalar
arguments. The damping is applied after the tridiagonal solve and before the
new geopotential is diagnosed.

## Memory and failure behavior

The old geopotential and old vertical momentum are both needed while the new
implicit solution is assembled, so one RHS field is irreducible without
recomputing sensitive expressions or changing the state contract. Rust makes
that 10.67 MiB benchmark workspace explicit and caller-owned. It is allocated
during setup and reused across timesteps. WRF additionally creates `wdwn`,
`msft_inv`, and `dampwt` automatic scratch; Rust evaluates those quantities
locally while preserving raw output bits.

Every field shape, coefficient length, lateral stencil, and complete-column
invariant is checked before state mutation. Validation failure therefore
leaves vertical momentum, geopotential, and time-averaged thermodynamics
unchanged. The workspace is intentionally unspecified after a call because it
is not model state.

## Direct parity and performance

The oracle extracts and compiles the pinned WRF v4.7.1 routine itself. Four
cases cover both transport discretizations, rigid and nonrigid tops, enabled
and disabled upper damping, global and specified/nested ranges, periodic X,
partial horizontal tiles, and inactive storage. All 2,592 state values match
by raw IEEE-754 bits. One- and four-worker results are identical.

On the matched 256 × 256 × 40 workload, optimized serial Fortran measured
16.745 ms. Rust measured 61.295 ms with one worker, 16.084 ms with four, and
6.621 ms with the standard 16-worker host pool. Four workers are already
within the same performance class; 16 workers are 2.53× faster, so SIMD work
is deferred.

See WRF-053 through WRF-056 in `UPSTREAM_FINDINGS.md` for dead interface
plumbing, hidden complete-column constraints, automatic scratch, and the
source's own unresolved map-factor/surface-condition comments.
