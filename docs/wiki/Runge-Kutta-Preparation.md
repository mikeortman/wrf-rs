# Runge-Kutta diagnostic preparation

WRF's Advanced Research WRF dynamical core advances prognostic fields with a
Runge-Kutta method. Before each step can assemble tendencies, `rk_step_prep`
derives a set of mutually consistent diagnostic fields from the current model
state. The wrapper is in `dyn_em/module_em.F`; its numerical routines are in
`dyn_em/module_big_step_utilities_em.F`.

This preparation is a dependency chain, not seven unrelated calculations:

```text
mu + mub ──> mut ───────────────────────────────> rw
    │
    ├──────> muu, muv ─────────────────────────> ru, rv
    ├──────> ww
moist ─────> cqu, cqv, cqw
al + alb ──> alt
ph + phb ──> php
```

`mut`, `muu`, and `muv` are immediately consumed by momentum coupling. The
other diagnostics feed later tendency and pressure-gradient calculations.
Computing them through one operation gives callers a clear consistency point:
either every diagnostic describes the same input state, or validation rejects
the pass before any output changes.

## The seven stages

### 1. Full dry-air column mass

`calculate_full` adds perturbation column mass `mu` to base-state mass `mub`.
Unlike most tile loops, it begins at `its-1` and `jts-1`. WRF communicates those
lower halo cells before the call because an IEVA path may consume them. The
Rust region validates this one-cell lower halo explicitly.

### 2. Mass on horizontal momentum points

`calc_mu_uv` averages `mu + mub` across adjacent mass points to obtain `muu` on
the west-east momentum stagger and `muv` on the south-north stagger. Physical
endpoints preserve WRF's duplicate-value averaging expression; periodic
endpoints use the communicated opposite-side halo.

### 3. Mass-coupled momentum

`couple_momentum` multiplies C-grid velocities by their local full dry-air mass
and divides by the relevant map factor. Half-level coefficients apply to the
horizontal components, while full-level coefficients apply to vertical
momentum. The three components have different upper-stagger clipping.

### 4. Dry-air omega

`calc_ww_cp` diagnoses eta-coordinate vertical mass flux from horizontal flux
divergence and integrates it through the complete column. It requires lower
horizontal neighbors, upper flux neighbors, and the top full level.

### 5. Moisture coefficients

`calc_cq` sums active Registry moisture species in declaration order, clips the
total at zero, and converts it into coefficients on all three momentum
staggers. Rust represents only active species; WRF's generated unused padding
slot is absent from the API.

### 6. Full inverse density

`calc_alt` adds perturbation inverse density `al` to base state `alb` over the
active mass tile. Inactive storage remains untouched by the Rust contract.

### 7. Geopotential at pressure points

`calc_php` averages base-state and perturbation geopotential from adjacent full
levels. The source's base-state-first four-term addition order is preserved so
exceptional single-precision behavior remains compatible.

## Rust ownership model

`RungeKuttaPreparationKernels` is a backend capability with one associated
native field type. The public request is divided into descriptive borrowed
groups:

- mass, velocity, map-factor, coefficient, moisture, and thermodynamic inputs;
- full/staggered mass, coupled momentum, and remaining diagnostic outputs; and
- the six independently validated region types required by the component
  geometries.

The groups allocate nothing. Rust prevents mutable output aliasing at the API
boundary. The CPU implementation validates every stage first, then invokes the
existing parity-tested kernels in WRF order. A future GPU backend can keep the
same public contract while fusing stages or retaining every field on device.

WRF's wrapper accepts `rk_step`, temperature, pressure arrays, two vertical
weights, and several map factors that none of these seven calls read. The Rust
boundary omits those dead arguments rather than making callers manufacture
irrelevant state.

## Failure atomicity

Component kernels already validate their own fields before mutation, but simple
sequential composition would not be enough: an invalid final `php` field could
be discovered after eleven earlier outputs had changed. The integrated CPU
implementation therefore performs an all-stage preflight. Tests deliberately
give the seventh stage an invalid output shape and verify that all twelve output
fields retain their sentinels.

This guarantee covers validation errors. A worker panic remains a stage-aware
execution error; as with most parallel numerical kernels, work completed by
other workers cannot be transactionally rolled back without copying complete
fields.

## Parity evidence

`scripts/run-runge-kutta-preparation-oracle.sh` extracts the exact seven pinned
WRF routine bodies and calls them in the wrapper's original order. The Rust
test recreates the same non-one memory origins, active bounds, two moisture
species, coefficients, map factors, and initial sentinels.

All 1,728 stored values across `mut`, `muu`, `muv`, `ru`, `rv`, `rw`, `ww`,
`cqu`, `cqv`, `cqw`, `alt`, and `php` match by raw IEEE-754 bits. This checks
both intermediate mass fields and final diagnostics, not merely an end-state
tolerance. One-worker and four-worker runs are also bitwise identical.

Remaining integration gates are a periodic coupled fixture, an exceptional
multi-stage corpus, Registry-backed state binding, and a short prognostic RK
trajectory.

## Performance and memory

On the matched 256 × 256 × 40 workload, optimized serial GNU Fortran measures
6.0671 ms. Release Rust measures 10.092 ms with one worker, 3.3025 ms with four,
and 4.5476 ms with 16. Four-worker Rust is 1.84× faster than serial WRF; the
default 16-worker path is 1.33× faster.

Settled execution uses 19 small scheduler allocations totaling 28,880 bytes per
100 calls, with no reallocations, numerical scratch, or field clones. The
ordinary parallel implementation clears the current performance gate, so no
cross-stage fusion or new SIMD specialization is justified before trajectory
profiling. See the [detailed baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/runge-kutta-preparation-2026-07-14.md).
