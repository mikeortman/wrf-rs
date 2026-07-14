# Dry-air omega diagnosis

WRF's `calc_ww_cp` diagnoses the dry-air eta-coordinate vertical mass flux
called `ww` from horizontal velocity and dry-air column mass. It is the fourth
diagnostic stage in `rk_step_prep`, after total column mass, staggered column
mass, and mass-coupled momentum. The pinned source is
`dyn_em/module_big_step_utilities_em.F`.

This `ww` is not pressure-coordinate meteorological omega in pascals per
second. It represents the mass-coupled vertical motion used by the ARW
eta-coordinate equations.

## Algorithm

For every active horizontal mass point, WRF first averages full dry-air column
mass `mup + mub` to west-east and south-north momentum faces:

```text
muu(i,j) = 0.5 * (mup(i,j) + mub(i,j) + mup(i-1,j) + mub(i-1,j))
muv(i,j) = 0.5 * (mup(i,j) + mub(i,j) + mup(i,j-1) + mub(i,j-1))
```

At half level `k`, define the west/east and south/north mass fluxes using
`c1h(k)`, `c2h(k)`, velocity, and their component-specific map factors. WRF
then evaluates

```text
divv(i,k,j) = msftx(i,j) * dnw(k) *
    (rdx * (east_flux - west_flux) +
     rdy * (north_flux - south_flux))
```

The vertically integrated column-mass tendency is

```text
dmdt(i,j) = sum_k divv(i,k,j)
```

With zero vertical flux at the bottom and top, internal full levels are
diagnosed bottom-up:

```text
ww(i,k,j) = ww(i,k-1,j)
            - dnw(k-1) * c1h(k-1) * dmdt(i,j)
            - divv(i,k-1,j)
```

The Rust implementation preserves each single-precision addition,
multiplication, division, subtraction, and vertical accumulation order. It
does not precombine column mass, replace division with reciprocals, or reassociate
the recurrence.

## Domain contract

Horizontal tiles may include one upper stagger point. Divergence is clipped to
the physical mass domain, while bottom/top boundary zeros retain WRF's complete
west-east tile. Every active point needs a lower mass neighbor for face
averaging and upper velocity/mass/map-factor neighbors for flux differences.

The vertical contract is stricter. WRF's derivation integrates over all eta
levels and imposes zero flux at both physical boundaries. The source accepts
`kts` but writes literal level `1` and begins its recurrence at literal level
`2`. Partial vertical tiles can therefore read uninitialized `divv` values or
write outside their declared logical tile. `OmegaDiagnosisRegion` requires the
complete half-level domain plus its top full-level face and rejects other
vertical tiles before output mutation.

## Rust API and backend boundary

`OmegaDiagnosisKernels` is the backend capability. Its CPU implementation uses:

- `OmegaDiagnosisVelocities` for `u` and `v`;
- `OmegaDiagnosisMasses` for perturbation and base-state column mass;
- `OmegaDiagnosisMapFactors` for the three factors actually read;
- `OmegaDiagnosisCoefficients` for `c1h`, `c2h`, and `dnw`;
- `OmegaDiagnosisGridMetrics` for `rdx` and `rdy`; and
- `OmegaDiagnosisRegion` for storage, domain, tile, neighbor, and complete-column
  validation.

All volume fields must share the region shape. Horizontal fields must match its
horizontal projection, and coefficient arrays must span allocated vertical
storage. Validation finishes before `ww` changes. Native field storage remains
behind the capability so a later GPU backend can provide device kernels rather
than accepting host closures.

WRF also passes `msfty`, `msfux`, `msfvx`, and `msfvy`, but `calc_ww_cp` never
reads them. The Rust API omits those signature-only arrays.

## Parallel and memory design

South-north planes are independent and run on the persistent default CPU pool.
Within a plane, divergence traverses equal-length west-east row views. The
nested `omega_diagnosis/row/` module names velocity, mass, map-factor,
coefficient, and output ownership so the hot loop stays safe and auditable
while allowing LLVM to remove repeated bounds checks.

The vertical recurrence remains column-local. The output's active half levels
temporarily store `divv`, and its top level temporarily stores `dmdt`; every
temporary is replaced by its final `ww` value before return. This removes WRF's
four automatic tile arrays without allocating numerical scratch.

## Parity evidence

`scripts/run-omega-diagnosis-oracle.sh` extracts and compiles the exact pinned
routine. Five cases cover interior tiles, each upper horizontal boundary,
combined boundaries, non-one and negative memory origins, complete top/bottom
levels, untouched storage, finite overflow, zero division, and a zero inverse
map factor.

All 1,960 stored output and sentinel values match. Finite values, infinities,
and signed zero compare by raw IEEE-754 bits; NaNs compare by class. Separate
tests cover all eight field roles, all three coefficient roles, each range
failure category, complete-column validation before mutation, and one/four
worker determinism.

WRF contains no dedicated numerical regression for this production routine.
A seeded randomized corpus and integrated `rk_step_prep` trajectory remain
later gates.

## Performance

On a matched 256 × 256 × 40 workload, serial optimized Fortran measured a
1.832250 ms median. Rust measured 5.0201 ms with one worker, 1.3306 ms with
four, and 666.90 µs with 16. Standard four-worker Rust is 1.38× faster than
serial Fortran; 16-worker Rust is 2.75× faster. Settled execution recorded one
1,520-byte scheduler allocation per 100 calls and no numerical scratch.

See the [detailed performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/omega-diagnosis-2026-07-13.md).
