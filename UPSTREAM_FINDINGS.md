# Upstream WRF findings

This ledger records findings against pinned WRF v4.7.1 commit
`f52c197ed39d12e087d02c50f412d90d418f6186`. It is written so each confirmed
item can be handed to WRF maintainers without depending on the Rust port.

Labels distinguish **confirmed bug**, **test gap**, and **performance
opportunity**. A performance observation is not presented as a measured
regression until a representative benchmark exists.

## Summary

| ID | Kind | Confidence | Area | Finding |
|---|---|---:|---|---|
| WRF-001 | Confirmed bug | Reproduced | ESMF time test | `Test1.F90` calls an obsolete dummy-argument keyword and does not compile against the bundled stub interface |
| WRF-002 | Performance opportunity | Source-confirmed, not benchmarked | Positive-definite correction | The sheet and slab routines allocate scratch storage and copy every corrected line into and out of it |
| WRF-003 | Performance opportunity | Source-confirmed, not benchmarked | Positive-definite correction | A full-array `ANY` scan is followed by a second `ANY` scan of each line |
| WRF-004 | Test gap | Repository search | Positive-definite correction | No dedicated regression test for either exported routine was found in the WRF tree |

## WRF-001: obsolete keyword in the bundled time test

Status: confirmed build bug.

`external/esmf_time_f90/Test1.F90:1181` calls:

```fortran
CALL ESMF_Initialize(defaultCalendar=ESMF_CAL_GREGORIAN, rc=rc)
```

The bundled procedure declares the optional dummy argument as
`defaultcalkind` in `ESMF_Stubs.F90:51-55`. Fortran keyword association uses
the dummy argument's name, so a compiler rejects `defaultCalendar`.

Reproduction:

1. preprocess `Test1.F90` with `TIME_F90_ONLY`;
2. build the bundled `libesmf_time.a`;
3. compile the generated test without modifying the keyword.

The local oracle changes only the generated build copy, from
`defaultCalendar=` to `defaultcalkind=`. With that one change, both the `ESMF_`
and `WRFU_` interfaces reproduce `Test1.out.correct` byte-for-byte.

Suggested upstream fix: update the keyword in `Test1.F90`, or intentionally
rename the bundled stub's dummy argument if the public compatibility name is
meant to be `defaultCalendar`.

## WRF-002: positive-definite scratch allocation and line copies

Status: source-confirmed performance opportunity; benchmark pending.

`dyn_em/module_positive_definite.F` allocates `line` once per routine call
(lines 21/32 and 80/87), copies each affected line into it (lines 37 and 91),
then copies the result back (lines 58 and 115). The allocation is not inside
the line loop, but the two full-width copies occur for every corrected line.

The Rust parity implementation mutates each disjoint contiguous line in place.
It removes both copies and the call-time allocation while producing the same
single-precision bit patterns for the current differential corpus.

Suggested upstream investigation: benchmark an in-place implementation on
representative `nx`, `ny`, correction frequency, compilers, and OpenMP/MPI
configurations. Alias analysis and existing caller expectations should be
checked before changing the Fortran routine.

## WRF-003: repeated negativity scans

Status: source-confirmed performance opportunity; benchmark pending.

The routines first evaluate `ANY(f < 0.)` over the full active array (lines 30
and 86). If true, each line is scanned again with another `ANY` (lines 36 and
90). On fields with at least one negative value, every line participates in
the global scan and then in a line scan.

The global scan may be beneficial when negative values are extremely rare
because it avoids allocation and line-loop setup. Whether it wins depends on
field size, cache behavior, correction frequency, and compiler lowering.

Suggested upstream investigation: benchmark the current two-stage scan against
a single per-line pass and against persistent/reused scratch storage. This is
an optimization candidate, not yet a claim of net slowdown.

## WRF-004: positive-definite test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the routine definitions and build dependency,
but no dedicated invocation from a test. The important uncovered branches are:

- an already nonnegative line, which must remain bit-for-bit unchanged;
- a negative supplied total, which zeros the line;
- minimum translation and total-preserving rescaling;
- the `sum_line <= 1.0e-15` degenerate branch;
- signed zero behavior;
- multiple independent lines; and
- slab indexing at domain, memory, and tile boundaries.

The first six are now exercised by the local Fortran oracle. The local slab
oracle additionally covers non-one memory origins, tile/domain clipping, and
unchanged halo/stagger sentinels; broader combinations remain worthwhile for
upstream regression coverage.
