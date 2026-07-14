# Acoustic pressure and inverse-density diagnosis

ARW's split-explicit integrator repeatedly reconstructs perturbation pressure
and inverse density while advancing fast acoustic modes. WRF v4.7.1 performs
this work in `dyn_em/module_small_step_em.F::calc_p_rho`, immediately after
`small_step_prep` and again at the end of each acoustic substep.

The Rust implementation exposes `AcousticPressureKernels`. It supports both
WRF governing-equation modes and both pressure-history phases without reducing
them to a single approximate formula.

## State and inputs

The routine updates four three-dimensional fields:

| Rust role | WRF name | Meaning |
|---|---|---|
| inverse-density perturbation | `al` | perturbation of inverse dry density |
| pressure perturbation | `p` | pressure variable used by the acoustic equations |
| geopotential perturbation | `ph` | updated only in hydrostatic mode |
| previous pressure | `pm1` | undamped pressure history |

Thermodynamic inputs are full inverse density `alt`, current acoustic
temperature `t_2`, reference temperature perturbation `t_1`, and the
linearized pressure coefficient `c2a`. Horizontal inputs are perturbation
column mass `mu` and full column mass `mut`.

Define, at half level `k`,

```text
mu'  = c1h(k) * mu(i,j)
M    = c1h(k) * mut(i,j) + c2h(k)
Tref = t0 + t_1(i,k,j)
```

The implementation stores these exactly rounded single-precision
subexpressions, preserving WRF operation order without recomputing or
reassociating them.

## Nonhydrostatic mode

Nonhydrostatic ARW diagnoses inverse density from the vertical geopotential
gradient:

```text
al = (-1 / M) * [alt * mu'
                 + rdnw(k) * (ph(k+1) - ph(k))]
```

It then applies the temporally linearized equation of state:

```text
p = c2a * [alt * (t_2 - mu' * t_1) / (M * Tref) - al]
```

`ph` is read but not changed. Rust computes `al` and `p` in one safe paired
output pass over contiguous X rows.

## Hydrostatic mode

Hydrostatic ARW first diagnoses pressure directly from column mass:

```text
p  = mu * c3h(k)
al = alt * (t_2 - mu' * t_1) / (M * Tref) - p / c2a
```

It then integrates geopotential upward:

```text
ph(k+1) = ph(k) - dnw(k) * [M * al + mu' * alt]
```

This is a true recurrence: level `k+1` becomes the input at the next level.
The CPU implementation therefore assigns one complete south-north plane to a
worker and visits levels in ascending order. Inside each level it traverses X
contiguously, matching WRF's cache-friendly `k`-then-`i` ordering. An initial
column-strided implementation was parity-correct but about four times slower;
the level-major form retained all oracle bits and removed that penalty.

## Pressure-history damping

The initialization call (`step == 0`) stores:

```text
pm1 = p
```

Later acoustic substeps apply forward pressure weighting:

```text
undamped = p
p        = p + smdiv * (p - pm1)
pm1      = undamped
```

The old pressure must be saved before mutation. Rust expresses initialization
and advance as `AcousticPressureDampingPhase` and updates the pressure/history
pair in one safe pass.

## Bounds and partial vertical tiles

WRF clips tile endpoints to `ide-1`, `jde-1`, and `kde-1`. The typed region
separates physical domain ranges from execution tiles and derives the clipped
mass-point rectangle. Every active half level must have allocated `ph(k+1)`
storage.

Unlike `small_step_prep`, this routine can operate on a partial vertical tile.
For hydrostatic mode the supplied `ph(k_start)` is the recurrence boundary; the
routine advances only the selected contiguous levels.

## Safe backend boundary

The public API groups WRF's positional arguments into mutable state,
thermodynamics, masses, coefficients, vertical metrics, scalar parameters,
governing mode, damping phase, and region. All ten field shapes, five vertical
array lengths, ranges, and upper-level access are checked before mutation.

The CPU backend uses the persistent default worker pool, disjoint mutable
blocks, and no numerical scratch or field clones. The capability trait owns an
associated native field type, leaving a GPU backend free to validate descriptors
and dispatch native kernels rather than accepting CPU closures.

## Intentional interface changes

The Rust API omits WRF arrays `c1f`, `c2f`, `c3f`, `c4f`, `c4h`, and `znu`, as
well as lower domain bounds `ids`, `jds`, and `kds`; the routine never reads
them. It preserves inactive caller storage explicitly instead of relying on the
undefined inactive portions of Fortran `INTENT(OUT)` arrays.

## Parity evidence

The oracle extracts the exact pinned `calc_p_rho` body and compares 3,456
complete records. Four cases cover both governing modes, initialization and
advance damping, full and partial tiles, hydrostatic recurrence, untouched
sentinels, zero denominators, infinities, and NaNs. Finite values and infinities
match by raw bits; NaNs match by class.

Additional Rust tests prove failure atomicity across all four mutable fields,
the `k+1` region contract, and one-versus-four-worker bit identity in both
modes. Performance and allocation evidence is in
`docs/performance/acoustic-pressure-2026-07-14.md`.

## Next integration gate

`calc_coef_w` consumes `c2a` and prepares the tridiagonal vertical solve used by
`advance_w`. Porting that routine completes the pre-loop coefficient boundary;
the subsequent gate is a complete acoustic substep through `advance_uv`,
`advance_mu_t`, and `advance_w` with state observed after each stage.
