# Positive-definite correction

## Purpose

Numerical transport can produce small negative values in fields whose physical
interpretation is nonnegative, such as a mixing ratio. WRF's
`positive_definite_sheet` corrects each west-east line independently while
scaling the corrected values to a caller-supplied line total.

This is a numerical repair operator, not a proof that every output is strictly
positive: zero is a normal output, and a degenerate line is filled with zero.

## Algorithm

Let a line contain values `x_i` and let `T` be its supplied total.

1. If every `x_i >= 0`, leave the line bit-for-bit unchanged. `T` is ignored.
2. If any value is negative and `T < 0`, fill the line with zero.
3. Otherwise compute `m = min_i(x_i)` and translate `y_i = x_i - m`.
4. Compute `S = sum_i(y_i)` in index order.
5. If `S > 10^-15`, output `z_i = y_i * T * (1 / S)`.
6. Otherwise fill the line with zero.

In exact arithmetic, the scaled line sums to `T`. In single-precision floating
point, division and multiplication rounding mean the recomputed sum can differ
slightly. The order `y_i * T * reciprocal` is intentionally retained.

## Rust implementation

The WRF routine allocates a scratch line and copies each corrected line into
and out of it. Rust partitions the field into disjoint mutable slices and
corrects them in place. Independent lines run through the persistent Rayon pool;
each line retains the upstream scalar order. This changes the implementation
and memory traffic while retaining exact output for the current corpus.

The reductions remain scalar because SIMD reduction trees may reassociate
addition. Translation and scaling are contiguous pointwise loops and may be
auto-vectorized; explicit SIMD requires generated-code inspection and a release
benchmark before adoption.

A `pulp` prototype later vectorized those pointwise loops with one runtime
dispatch per kernel and preserved exact bits across vector/tail boundaries. It
was removed because normal Criterion measurements slowed the representative
one- and four-worker cases. The scalar implementation is intentionally the
current optimized choice; the experiment and measurements are retained in the
performance baseline.

## Three-dimensional slab variant

`positive_definite_slab` applies the same translation-and-rescaling idea to
selected west-east lines in a three-dimensional `f(i, k, j)` field. Unlike the
sheet routine, it derives the target total from the original line rather than a
separate array and has no epsilon guard. A line whose original sum is negative
is zeroed; a nonnegative line is left unchanged.

The Fortran call expresses domain, memory, and tile bounds as 18 inclusive
indices with potentially non-one lower bounds. The Rust API converts those at
the caller boundary into `PositiveDefiniteSlabRegion`: validated, zero-based,
half-open ranges tied to a particular `GridShape`. This makes empty and
out-of-bounds regions typed errors. The numerical kernel still visits
first-index-contiguous lines and leaves halos, staggered endpoints, and clipped
tile storage untouched.

## Edge semantics and evidence

The differential oracle compiles
`dyn_em/module_positive_definite.F` directly with only an empty error-module
stub. Its fixtures cover unchanged lines, negative totals, ordinary
translation/scaling, equal-value degeneracy, multiple lines, the epsilon
branch, positive and negative zero totals, and preservation of signed zero on
an unchanged line.

The Rust suite additionally proves sheet bitwise equality between one and four
CPU workers and rejects incompatible shapes or totals. A slab fixture exercises
non-one Fortran memory origins, domain/tile clipping, all major correction
branches, and untouched halo/stagger sentinels. NaN and infinity behavior,
randomized differential fixtures, and broader slab-bound combinations remain
documented gaps.
