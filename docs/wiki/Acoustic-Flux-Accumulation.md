# Acoustic flux accumulation

## Role in ARW

WRF calls `sumflux` after vertical momentum in every acoustic substep and
before the closing pressure diagnosis. It accumulates the nonlinear mass
fluxes that scalar transport later consumes. On the last substep it converts
the sums to arithmetic means and restores the saved large-step linear flux.

The three fields occupy different C-grid staggers:

- `ru_m` includes the upper west-east stagger point;
- `rv_m` includes the upper south-north stagger point;
- `ww_m` includes the upper full vertical level.

All three are cleared across the entire supplied tile on iteration one. This
includes stagger points that a particular field does not subsequently
accumulate. Preserving those zero stores matters for complete-field parity.

## Equations

For substep count `N`, each active running sum receives the current nonlinear
flux. On iteration `N`, WRF finalizes the fields as

```text
ru_m = ru_m / N + (c1h * muu + c2h) * u_lin / msfuy
rv_m = rv_m / N + (c1h * muv + c2h) * v_lin * msfvx_inv
ww_m = ww_m / N + ww_lin
```

The Rust implementation retains this operation order, including division by
the single-precision conversion of `N`. It does not replace division with a
precomputed reciprocal because that can change IEEE-754 rounding.

## Safe Rust boundary

`AcousticSubstepPhase` rejects zero counts and iterations outside `1..=N`.
`AcousticFluxAccumulationRegion` separates physical mass/half-level domains
from a tile that may include one upper stagger point per axis. Borrowed field
groups distinguish current nonlinear fluxes, saved linear fluxes, staggered
column masses, map factors, and mutable averages.

WRF passes ten arrays/scalars that `sumflux` never reads. The Rust capability
omits them. Shape and coefficient validation completes before the first mutable
store, so caller errors are failure-atomic.

## Parallel and memory behavior

Each average field owns an independent output, so the CPU backend processes
complete south-north planes in parallel without aliases, locks, or numerical
scratch. First-substep clearing is a separate contiguous pass. No field is
cloned, and no tile-sized temporary is allocated.

On the matched 256 × 256 × 40 three-substep workload, portable 16-worker Rust
is 1.52× faster than optimized serial Fortran. The serial Rust gap is not being
tuned in isolation because the standard parallel path clears the project gate.

## Parity evidence

The oracle extracts the exact WRF v4.7.1 `sumflux` body, executes three
substeps, and compares every stored bit of `ru_m`, `rv_m`, and `ww_m`: 375
values including halos, inactive sentinels, and zeroed stagger-only points.
Added tests cover invalid phases, coefficient failure atomicity, typed field
shape errors, and multithreaded execution.

The next trajectory gate composes this kernel with preparation, pressure,
coefficient construction, horizontal momentum, mass/theta, and vertical
momentum in the order used by `solve_em.F`.
