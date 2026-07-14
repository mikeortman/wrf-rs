# Held-Suarez momentum damping

## Purpose and model context

The Held-Suarez benchmark is an idealized dry-atmosphere experiment used to
evaluate the dynamical behavior of atmospheric circulation models without the
complexity of a full physical-parameterization suite. WRF's
`held_suarez_damp` routine contributes a pressure-dependent Rayleigh-friction
term to the horizontal momentum tendencies. It damps momentum most strongly
near the surface and applies no damping above a prescribed normalized-pressure
boundary.

This port covers the damping routine in
`dyn_em/module_damping_em.F`; it does not yet constitute the complete
Held-Suarez test case or an end-to-end ARW trajectory.

## Equations

For either horizontal momentum component, WRF computes a staggered normalized
pressure

```text
sigma = staggered total pressure at level k
        -----------------------------------
        staggered total pressure at level 1
```

where total pressure is perturbation pressure `p` plus base pressure `pb`. The
dimensionless vertical envelope is

```text
sigma_term = max(0, (sigma - 0.7) / (1 - 0.7)).
```

The tendency update is

```text
momentum_tendency -= (1 / 86400 seconds) * sigma_term * momentum.
```

Consequently, points with `sigma <= 0.7` are unchanged. The damping timescale
at `sigma = 1` is one day. The Rust implementation retains single precision,
the upstream parenthesization, and the multiply/subtract order so current
fixtures match raw IEEE-754 bits.

## Arakawa-C staggering

WRF stores the west-east momentum (`ru`) and south-north momentum (`rv`) on
different horizontal faces. Pressure is mass-point data, so each component
uses the two pressure points adjacent to its face:

- `ru(i,k,j)` averages pressure at `i-1` and `i`;
- `rv(i,k,j)` averages pressure at `j-1` and `j`.

The Rust `HeldSuarezDampingRegion` validates that both active staggered ranges
have the required preceding neighbor. It also converts Fortran's inclusive,
possibly non-one-origin bounds into zero-based half-open memory offsets before
the numerical loop begins.

## Storage and parallel execution

Fields use contiguous WRF order `f(i, k, j)`, with `i` varying fastest. Each
complete west-east line is an independent mutable output block. The persistent
Rayon pool schedules those blocks while pressure and momentum fields are shared
immutably. There are two scheduling passes because WRF updates two distinct
tendency fields over slightly different south-north ranges.

The field bundle borrows six domain-sized fields; it neither clones nor
allocates them. Shape validation finishes before either tendency is mutated.
The hot loops allocate no point or line scratch storage.

`pulp` performs safe runtime SIMD selection once around the complete kernel.
Each worker then applies the same ordered single-precision formula to contiguous
lane groups, followed by a scalar tail. SIMD is a computational layer beneath
the backend capability; callers and future GPU implementations do not depend on
its types.

## Parity evidence and added tests

The differential oracle compiles the pinned upstream
`module_damping_em.F` directly. Its fixture uses non-one memory origins and
checks raw bits at 16 selected points spanning:

- both staggered momentum components;
- active surface, partially damped, and undamped levels;
- lower and upper vertical exclusions;
- domain/tile-clipped south-north rows; and
- west-east points outside the active range.

Rust additionally checks one-versus-four-worker bitwise determinism, validates
all region ranges and staggered neighbors, rejects mismatched field shapes
before mutation, and checks the pressure reference level. A 1–257-length corpus
also compares runtime SIMD with the scalar implementation by raw bits.

Twelve seeded cases add 19,698 complete tendency outputs across varying shapes,
negative/non-one memory origins, clipped tiles, both C-grid staggers, signed
zero, large finite momentum, and active NaN/infinity momentum. Finite values and
infinities compare by raw bits; NaN compares by class. The default host-parallel
and safe runtime-SIMD implementation matches the pinned Fortran corpus. A full
Held-Suarez trajectory remains an open integration item.
