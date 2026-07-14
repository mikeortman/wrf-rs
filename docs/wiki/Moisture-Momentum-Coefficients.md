# Moisture momentum coefficients

WRF's `calc_cq` prepares moisture-dependent coefficients for the three ARW
momentum equations. It is the fifth diagnostic stage in `rk_step_prep`, after
column mass, staggered mass, mass-coupled momentum, and dry-air omega. The
pinned implementation is in `dyn_em/module_big_step_utilities_em.F`.

The routine does not advance moisture. It reads the current mixing-ratio fields
and maps their total moist loading from mass points to the west-east,
south-north, and vertical momentum locations used later in the pressure-gradient
and small-step equations.

## Species model

WRF Registry generation defines `PARAM_FIRST_SCALAR = 2`. Scalar-array slot 1
is reserved infrastructure; physical moisture fields occupy slots 2 through
`n_moist`. Let

```text
Q(i,k,j) = sum over active species s of moist(i,k,j,s)
```

where the sum follows Registry order. Typical fields include water vapor and
hydrometeor mixing ratios selected by the configured physics, but `calc_cq`
does not interpret species names. Every active field contributes equally to
the total.

The exact accumulation order matters in single precision. WRF updates one
running total per point for species 2, then species 3, and so on. Rust accepts
an ordered slice containing only active fields and preserves that order. It
does not expose or allocate the reserved slot.

## C-grid equations

For a west-east momentum point, WRF averages moisture from the current and
western mass cells, then stores the reciprocal moist-mass factor:

```text
Q_u(i,k,j) = 0.5 * sum_s (q_s(i,k,j) + q_s(i-1,k,j))
cqu(i,k,j) = 1 / (1 + Q_u(i,k,j))
```

The south-north coefficient uses the current and southern mass cells:

```text
Q_v(i,k,j) = 0.5 * sum_s (q_s(i,k,j) + q_s(i,k,j-1))
cqv(i,k,j) = 1 / (1 + Q_v(i,k,j))
```

At an internal vertical momentum face, the total is averaged between adjacent
half levels:

```text
cqw(i,k,j) = 0.5 * sum_s (q_s(i,k,j) + q_s(i,k-1,j))
```

`cqw` is intentionally not inverted here. Later WRF dynamics code derives
complementary factors from it and may replace its stored representation before
the small-step solver consumes it. The Rust port reproduces only the contract
of `calc_cq` at this boundary.

When there are no active species, WRF skips all moisture reads and writes the
dry limits directly:

```text
cqu = 1
cqv = 1
cqw = 0
```

## Stagger and tile contract

Physical domain ranges describe mass points and exclude their upper stagger.
The active tile may include one upper point on every axis. WRF applies a
different clipping rule to each output:

| Output | West-east extent | South-north extent | Bottom-top extent |
|---|---|---|---|
| `cqu` | complete tile, including upper U stagger | clipped to mass domain | clipped to half-level domain |
| `cqv` | clipped to mass domain | complete tile, including upper V stagger | clipped to half-level domain |
| `cqw` | clipped to mass domain | clipped to mass domain | starts one level above `kts`, clipped at domain top |

Active `cqu` points require a stored western neighbor. Active `cqv` points
require a stored southern neighbor. `cqw` starts at `kts + 1`, so its lower
vertical neighbor is inside the supplied tile. When active species exist,
`MoistureCoefficientRegion` and the kernel check these conditions before any
output changes. The dry branch does not require neighbors it never reads.

WRF declares all three outputs `INTENT(OUT)` but assigns only the component
ranges above. The Rust API deliberately preserves inactive storage because
callers own those fields; the exact oracle verifies every untouched sentinel.

## Rust API and backend boundary

`MoistureCoefficientKernels` is the focused backend capability.

- `MoistureCoefficientOutputs` owns three distinct mutable borrows, so safe
  Rust prevents output aliasing at compile time.
- `MoistureSpecies` borrows an ordered slice of active backend-native fields.
- `MoistureCoefficientRegion` owns shape, physical-domain, tile, clipping, and
  lower-neighbor validation.
- `MoistureCoefficientError` distinguishes axis/range, output-role, species
  index, and worker failures.

All output and species fields must share the region's shape. Validation of all
fields completes before mutation. The capability exposes field storage rather
than CPU closures, so a future GPU backend can keep species and coefficients on
device and implement the same operation natively.

## Parallel and memory design

Each `(j,k)` west-east row is independent for a given component. The CPU backend
runs rows through the persistent default worker pool in three component passes.
Within a row it:

1. clears only the output range;
2. accumulates each active species into that output row in WRF order; and
3. transforms the total in place into the final coefficient.

The active output row therefore replaces WRF's automatic `qtot(its:ite)`
scratch. It is fully overwritten before return, requires no numerical
allocation, and retains contiguous access across every species field. The
implementation is split under `moisture_coefficients/cpu/` by west-east,
south-north, and vertical ownership rather than putting all loops in one flat
file.

## Parity evidence

`scripts/run-moisture-coefficient-oracle.sh` extracts and compiles the exact
pinned routine with WRF's generated `PARAM_FIRST_SCALAR = 2` value. Seven cases
cover:

- zero, one, and three active species;
- a poisoned reserved slot proving that it is ignored;
- interior tiles and each upper horizontal stagger;
- combined horizontal and vertical upper clipping;
- negative and non-one Fortran memory origins;
- every inactive output and halo sentinel;
- signed zero and finite inputs that overflow during accumulation; and
- deterministic one- versus four-worker execution.

All 8,232 stored `cqu`, `cqv`, `cqw`, and sentinel values match. Finite values,
infinities, and signed zero compare by raw IEEE-754 bits; NaNs compare by class.
Separate tests cover every range category, output role, species index, and
validation-before-mutation behavior.

WRF has no dedicated numerical regression for the production routine. A seeded
randomized corpus, Registry-generated species binding, and an integrated
`rk_step_prep` trajectory remain future gates.

## Performance

On a matched 256 × 256 × 40 workload with six active species, optimized serial
Fortran measured a 5.221150 ms median. Rust measured 7.1239 ms with one worker,
2.0418 ms with four, and 3.4246 ms with 16. Four-worker Rust is 2.56× faster;
the all-core path is 1.52× faster. Every 100 calls recorded five 1,520-byte
scheduler allocations and no numerical scratch.

The standard multithreaded implementation is already competitive, so no
explicit SIMD or custom worker policy is justified without integrated profile
evidence. See the [detailed performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/moisture-coefficients-2026-07-13.md).
