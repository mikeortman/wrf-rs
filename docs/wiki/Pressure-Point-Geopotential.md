# Pressure-point geopotential

WRF's `calc_php` derives full geopotential at ARW pressure, or mass, points from
geopotential stored on adjacent vertical full levels. It is the seventh and
final diagnostic call currently made by `rk_step_prep`. The pinned routine is
in `dyn_em/module_big_step_utilities_em.F`.

## Geometric role

ARW stores perturbation geopotential `ph` and hydrostatic base-state
geopotential `phb` on vertically staggered full levels. Many horizontal
pressure-gradient calculations need total geopotential at the mass point
between two adjacent full levels. WRF calculates:

```text
php(i,k,j) = 0.5 * (
    phb(i,k,j) + phb(i,k+1,j)
  + ph(i,k,j)  + ph(i,k+1,j)
)
```

The result remains geopotential, normally measured in square metres per square
second. It is not divided by gravitational acceleration here, so `php` should
not be confused with geometric height.

The single-precision operation order is part of the compatibility contract.
WRF adds the two base-state terms first, then the current perturbation, then the
upper perturbation, and finally multiplies by `0.5`. The Rust kernel preserves
those steps. This matters for overflow and cancellation even though ordinary
atmospheric values are finite and well scaled.

## Domain and vertical-neighbor contract

WRF accepts independent physical domain, allocated memory, and active tile
bounds. It clips the active upper endpoint on every axis to one below the
physical upper boundary:

```text
itf = min(ite, ide - 1)
jtf = min(jte, jde - 1)
ktf = min(kte, kde - 1)
```

The current output level `k` also reads `k + 1`. Therefore the input fields
must store a full level at the exclusive upper end of the active mass-level
domain. WRF relies on the relationship between `kde` and `kme` without checking
it. `PressurePointGeopotentialRegion` validates the upper neighbor explicitly
before mutation.

Rust ranges are zero-based and half-open storage offsets. A tile may include
one upper stagger point on each axis, but output is clipped to the mass domain.
Negative and non-one Fortran origins map to offsets without changing the
geometry. Points outside the clipped tile are preserved.

## Rust API and ownership

`PressurePointGeopotentialKernels` is a focused backend capability. Its method
borrows:

- mutable pressure-point full geopotential output;
- immutable full-level perturbation geopotential;
- immutable full-level base-state geopotential; and
- a validated `PressurePointGeopotentialRegion`.

`PressurePointGeopotentialField` gives each field role a typed validation
identity. All three shapes and the vertical-neighbor contract are checked
before any output changes. Rust's borrow rules prevent output/input aliasing.
The trait works with backend-native storage, leaving a clean boundary for a
future GPU implementation that keeps the arrays on device.

## CPU execution and memory

The workspace stores fields in contiguous west-east rows for each `(j,k)`.
Each output row is independent. The persistent CPU pool owns disjoint mutable
rows while all four input row views are immutable:

- current and upper base-state rows;
- current and upper perturbation rows.

The inner loop zips equal-length slices, preserving source order without
general three-dimensional indexing. It uses no unsafe code, numerical scratch,
or per-point allocation. The same readable layout exposes normal compiler
autovectorization.

## Parity evidence

`scripts/run-pressure-point-geopotential-oracle.sh` extracts and compiles the
exact pinned `calc_php` body. Six complete-storage cases cover:

- an interior tile;
- independent west-east, south-north, and bottom-top upper clipping;
- all upper boundaries together;
- negative and non-one memory origins;
- the required upper full-level neighbor;
- untouched halos and stagger sentinels;
- source-order-sensitive finite overflow, signed zero, and opposite infinities;
- every domain, tile, field-role, and missing-neighbor validation path; and
- bitwise one-worker versus four-worker determinism.

All 2,352 output and sentinel values match pinned Fortran. Finite values,
infinities, and signed zero compare by raw IEEE-754 bits; NaNs compare by class.
WRF has no dedicated numerical regression for the production routine. A seeded
randomized corpus and integrated `rk_step_prep` trajectory remain future gates.

## Performance

On the matched 256 × 256 × 40 workload, optimized GNU Fortran measured a
0.402140 ms median. Rust measured 0.44482 ms with one worker, 0.14072 ms with
four, and 0.40852 ms with 16. Serial Rust is 10.6% slower; four-worker Rust is
2.86× faster.

Settled execution records one 1,520-byte scheduler allocation per 100 calls,
with no reallocations or numerical scratch. Ordinary multithreaded Rust already
clears the performance gate, so no explicit SIMD specialization is added. See
the [detailed performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/pressure-point-geopotential-2026-07-14.md).
