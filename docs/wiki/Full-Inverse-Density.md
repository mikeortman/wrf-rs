# Full inverse density

WRF's `calc_alt` reconstructs total inverse dry-air density on ARW mass points.
It is the sixth diagnostic stage of `rk_step_prep`, after column mass,
mass-coupled momentum, dry-air omega, and moisture coefficients. The pinned
routine is in `dyn_em/module_big_step_utilities_em.F`.

## Physical role

ARW splits inverse density into a hydrostatically balanced base state and a
time-varying perturbation:

```text
alpha_total(i,k,j) = alpha_perturbation(i,k,j) + alpha_base(i,k,j)
```

WRF names these fields `alt`, `al`, and `alb`. Inverse density is specific
volume: ordinary dry-air density is approximately `1 / alt`. Keeping the base
state separate lets the dynamical core evolve smaller perturbations while
retaining the complete state wherever pressure-gradient, diffusion, or
diagnostic equations need it.

`calc_alt` performs no interpolation, averaging, reciprocal, or moisture
correction. Moisture-dependent density conversions occur elsewhere. This
routine is deliberately just the single-precision addition above, in the same
operand order as WRF.

## Domain, memory, and tile semantics

WRF passes three sets of inclusive Fortran bounds:

- physical domain (`ids:ide`, `jds:jde`, `kds:kde`);
- allocated memory (`ims:ime`, `jms:jme`, `kms:kme`); and
- active tile (`its:ite`, `jts:jte`, `kts:kte`).

The upper physical bounds describe stagger-aware grid storage. `calc_alt`
computes mass points only, so it clips every active upper endpoint to one below
the corresponding physical upper bound:

```text
itf = min(ite, ide - 1)
jtf = min(jte, jde - 1)
ktf = min(kte, kde - 1)
```

Lower tile endpoints are used directly. The Rust `InverseDensityRegion` stores
zero-based half-open equivalents of the same shape, domain, and tile ranges.
It validates that a tile remains inside the mass domain plus at most one upper
stagger point, then derives the clipped output range. Negative or non-one
Fortran origins therefore become ordinary storage offsets without losing their
geometric meaning.

Storage outside the clipped active tile is not calculated. WRF declares `alt`
as `INTENT(OUT)` even though it only writes that subset; the Rust API explicitly
preserves inactive caller-owned storage. The oracle checks every sentinel.

## Rust API

`InverseDensityKernels` is the backend capability. Its method accepts:

- one mutable backend-native full inverse-density field;
- immutable perturbation and base-state fields; and
- a validated `InverseDensityRegion`.

All three shapes are checked before mutation. `InverseDensityField` identifies
the exact role in a typed mismatch error. Distinct Rust borrows prevent output
aliasing with either input. The capability is defined over backend field
storage, so a future GPU backend can keep all three arrays on device and launch
its own native kernel.

The CPU implementation lives under `inverse_density/cpu/`. Each `(j,k)` row is
independent and contiguous in the workspace's WRF-compatible storage order.
The persistent CPU pool assigns disjoint mutable output rows while inputs are
shared immutably. The inner loop adds equal-length slices without indexing
through a general three-dimensional accessor, numerical scratch, unsafe code,
or per-point allocation.

## Numerical behavior

The exact routine body is extracted and compiled by
`scripts/run-inverse-density-oracle.sh`. Six complete-storage cases cover:

- an interior tile;
- independent west-east, south-north, and bottom-top upper clipping;
- all upper boundaries together;
- negative and non-one Fortran memory origins;
- untouched halos and stagger sentinels;
- finite overflow, exact cancellation, signed zero, and opposite infinities;
- all domain/tile validation categories and every field role; and
- bitwise one-worker versus four-worker determinism.

All 2,352 stored output and sentinel values match pinned Fortran. Finite values,
infinities, and signed zero require exact IEEE-754 bits. NaNs compare by class
because payload propagation is not a portable atmospheric-data contract.

WRF has no dedicated production-routine regression for `calc_alt`. A seeded
randomized corpus and an integrated `rk_step_prep` trajectory remain future
gates.

## Performance

On the matched 256 × 256 × 40 workload, GNU Fortran 16.1.0 `-O3 -flto`
measured a 0.210880 ms median. Rust measured 0.32594 ms with one worker,
0.12102 ms with four, and 0.39732 ms with 16. Four-worker Rust is 1.74× faster
than serial Fortran.

Settled execution records one 1,520-byte scheduler allocation per 100 calls,
with no reallocations or numerical scratch. The contiguous loop is already
compiler-vectorizable and the normal multithreaded path is competitive, so no
explicit SIMD specialization is justified. See the
[detailed performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/inverse-density-2026-07-14.md).
