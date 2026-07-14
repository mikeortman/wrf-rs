# Vertical acoustic solve coefficients

WRF's `calc_coef_w` prepares a tridiagonal system for the vertically implicit
part of each nonhydrostatic acoustic step. The Rust port exposes this operation
through `VerticalAcousticCoefficientKernels` and preserves the pinned WRF
v4.7.1 arithmetic exactly.

## Why the solve is implicit

Vertically propagating acoustic and gravity waves impose a much smaller stable
explicit time step than the horizontally resolved meteorological flow. ARW
uses small acoustic steps and treats the vertical coupling implicitly. After
the vertical momentum and geopotential equations are combined, each horizontal
column has a tridiagonal linear system:

```text
a[k] w[k-1] + b[k] w[k] + c[k] w[k+1] = r[k]
```

Only adjacent full levels interact, so the system can be solved in linear time
with the Thomas algorithm. `calc_coef_w` performs the coefficient construction
and forward factorization once for a Runge–Kutta predictor. WRF reuses the
result during the corresponding corrector step.

## Forward factorization

WRF stores three arrays:

- `a`: the lower-diagonal coefficient;
- `alpha`: the reciprocal diagonal after eliminating the preceding level; and
- `gamma`: the normalized upper coefficient.

Starting above the lower boundary, the recurrence is:

```text
alpha[k] = 1 / (b[k] - a[k] gamma[k-1])
gamma[k] = c[k] alpha[k]
```

The later `advance_w` routine uses the factors in two sweeps:

```text
w[k] = (w[k] - a[k] w[k-1]) alpha[k]
w[k] = w[k] - gamma[k] w[k+1]
```

This dependency is vertical within a column. Different horizontal columns are
independent.

## Physical coefficients

Define the acoustic scaling factor

```text
q = (0.5 dts g (1 + epssm))²
```

where `dts` is the acoustic time step, `g` is gravitational acceleration, and
`epssm` is WRF's vertical off-centering control. Hybrid-coordinate masses are

```text
Mh[k] = c1h[k] mut + c2h[k]
Mf[k] = c1f[k] mut + c2f[k]
```

for half and full levels. `mut` is full dry column mass. The pressure
coefficient `c2a`, inverse eta spacings `rdn` and `rdnw`, and moisture correction
`cqw` form the lower, diagonal, and upper terms. The Rust code deliberately
retains WRF's single-precision expression order; algebraically equivalent
factorization can change rounding and a model trajectory.

At the lower boundary, WRF sets `a[2] = 0` and `gamma[1] = 0`. Interior levels
use the full tridiagonal recurrence. The top level has `c = 0`, so its computed
`gamma` is normally zero. The rigid-lid mode also multiplies the top `a` term
by zero. The port performs the literal multiplication rather than branching,
because `0 × infinity` and `0 / 0` are observable NaNs in the upstream routine.

## Vertical and horizontal ranges

The routine clips west–east and south–north tiles to their mass domains. It
does not use the caller's vertical tile bounds: it always constructs the
complete physical column from the lower full level through the extra top full
level. `VerticalAcousticCoefficientRegion` models that fact directly:

- horizontal domain and tile ranges are distinct and validated;
- the supplied half-level range is the complete physical mass column; and
- one additional full-level storage point is required at its exclusive end.

This makes WRF's implicit assumption explicit and prevents a caller from
mistaking a partial vertical tile for a supported operation.

## Rust ownership and execution

The public interface separates:

- `VerticalAcousticSolveCoefficients` for the three non-aliasing outputs;
- `VerticalAcousticCoefficientInputs` for mass, moisture, and pressure fields;
- `VerticalAcousticMassCoefficients` and `VerticalAcousticMetrics` for borrowed
  one-dimensional coordinate arrays;
- `VerticalAcousticCoefficientParameters` for scalar controls; and
- `VerticalAcousticTopBoundary` for the top-boundary policy.

Every shape, range, upper-level, and coefficient-length contract is checked
before mutation. The CPU backend then runs two safe phases: lower-diagonal
construction followed by paired `alpha`/`gamma` elimination. South–north
planes are disjoint parallel work units. Within each plane, levels are outer
and west–east points are inner, matching XZY storage and WRF's contiguous loop
direction. There are no field clones, numerical scratch arrays, unsafe blocks,
or per-column allocations.

The capability trait owns the scientific contract rather than accepting a CPU
closure. A future GPU implementation can retain the same typed inputs and
outputs while using device-native fields and a column kernel.

## Parity evidence

`scripts/run-vertical-acoustic-coefficients-oracle.sh` extracts the exact
pinned `calc_coef_w` body, compiles it with GNU Fortran, and compares a committed
3,024-value golden against Rust. The cases cover:

- nonrigid and rigid-lid top boundaries;
- full and partial horizontal tiles;
- negative and non-one Fortran memory origins;
- the complete-column behavior despite degenerate vertical tile bounds;
- inactive storage sentinels;
- signed zero, zero denominators, infinities, and NaN classification;
- failure before any of the three outputs mutate; and
- bit-identical one-worker and four-worker execution.

Finite values, signed zero, and infinities require raw IEEE-754 equality. NaNs
require class equality because payload propagation is not a portable model
contract.

## Performance

On the matched 256 × 256 × 40 mass-column workload, optimized serial Fortran
measures 1.867500 ms. Rust measures 14.608 ms with one worker, 3.8912 ms with
four, and 1.7109 ms with the standard 16-worker host pool. The default path is
1.09× faster than serial Fortran.

The first exact Rust traversal was column-strided and measured 27.881 ms
serially. Switching to level-major contiguous-X traversal preserved all oracle
bits and reduced that time by 47.6%. The remaining serial gap is a candidate
for safe SIMD only if an integrated acoustic trajectory identifies this stage
as a material bottleneck. Three 1,520-byte scheduler allocations occur per 100
calls, with no reallocations or numerical scratch.
