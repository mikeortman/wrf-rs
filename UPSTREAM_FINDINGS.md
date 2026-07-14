# Upstream WRF findings

This ledger records findings against pinned WRF v4.7.1 commit
`f52c197ed39d12e087d02c50f412d90d418f6186`. It is written so each confirmed
item can be handed to WRF maintainers without depending on the Rust port.

Labels distinguish **confirmed bug**, **test gap**, **numerical robustness**,
and **performance opportunity**. A performance observation is not presented as
a measured regression until a representative benchmark exists.

## Summary

| ID | Kind | Confidence | Area | Finding |
|---|---|---:|---|---|
| WRF-001 | Confirmed bug | Reproduced | ESMF time test | `Test1.F90` calls an obsolete dummy-argument keyword and does not compile against the bundled stub interface |
| WRF-002 | Performance opportunity | Matched routine benchmark; combined attribution | Positive-definite correction | The sheet and slab routines allocate scratch storage and copy every corrected line into and out of it |
| WRF-003 | Performance opportunity | Matched routine benchmark; combined attribution | Positive-definite correction | A full-array `ANY` scan is followed by a second `ANY` scan of each line |
| WRF-004 | Test gap | Repository search | Positive-definite correction | No dedicated regression test for either exported routine was found in the WRF tree |
| WRF-005 | Test gap | Repository search | Held-Suarez damping | No dedicated numerical regression for `held_suarez_damp` was found in the WRF tree |
| WRF-006 | Performance opportunity | Source-confirmed, not benchmarked | Held-Suarez damping | The surface-pressure denominator is recomputed for every vertical level although it is invariant in `k` |
| WRF-007 | Test gap | Repository search | Column-mass staggering | No dedicated numerical regression for `calc_mu_staggered` was found in the WRF tree |
| WRF-008 | Numerical robustness | Reproduced against pinned Fortran | Positive-definite correction | Finite extreme inputs can overflow the intermediate scale multiplication and produce infinity even when the normalized result is representable |
| WRF-009 | Confirmed bug | Compiler-diagnosed and source-reproduced | Registry allocation generator | Empty output-directory branches pass a string to `%d` and an unused integer to `sprintf`, invoking undefined behavior |
| WRF-010 | Confirmed bug | Reproduced for non-one origins | RSL_LITE decomposition | `task_for_point` mixes absolute indices and offsets, lacks a return, and has malformed diagnostic formatting |
| WRF-011 | Numerical consistency | Source-confirmed against default constants | Kessler microphysics | Saturation adjustment uses passed `xlv`/`cp`, while latent heating and pressure use different hard-coded constants |
| WRF-012 | Confirmed latent bug | Source-confirmed; bounds-check reproducer recommended | Kessler microphysics | Several sedimentation loops index from literal level `1` despite accepting and allocating from `kts` |
| WRF-013 | Test gap | Repository search | Kessler microphysics | No dedicated numerical regression for the exported Kessler scheme was found in the WRF tree |

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

Status: source-confirmed performance opportunity with a matched routine
benchmark; individual cost not isolated.

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

On the local 256 × 4,096-line all-correction workload, one-worker Rust is 1.48×
faster than optimized serial Fortran for the sheet and 1.29× faster for the
slab. Rust removes the scratch allocation and copies, but also removes the
global `ANY` scan and uses a different compiler. The result supports upstream
investigation but does not attribute the gain to line copies alone.

## WRF-003: repeated negativity scans

Status: source-confirmed performance opportunity with a matched routine
benchmark; individual cost not isolated.

The routines first evaluate `ANY(f < 0.)` over the full active array (lines 30
and 86). If true, each line is scanned again with another `ANY` (lines 36 and
90). On fields with at least one negative value, every line participates in
the global scan and then in a line scan.

The global scan may be beneficial when negative values are extremely rare
because it avoids allocation and line-loop setup. Whether it wins depends on
field size, cache behavior, correction frequency, and compiler lowering.

Suggested upstream investigation: benchmark the current two-stage scan against
a single per-line pass and against persistent/reused scratch storage. This is
an optimization candidate. The matched benchmark reported under WRF-002 shows
a combined routine-level gain but does not isolate the scan from scratch-copy
and compiler effects.

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

## WRF-005: Held-Suarez damping test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the routine definition, its production call
from `dyn_em/module_em.F`, build dependencies, and commented tangent-linear or
adjoint references. It finds no dedicated numerical test invocation. Important
uncovered behavior includes:

- both C-grid momentum staggers and their preceding pressure neighbors;
- the `sigma = 0.7` zero-damping boundary;
- surface, partially damped, and unchanged upper levels;
- clipping of domain and tile bounds; and
- unchanged halo and excluded vertical points.

The local differential oracle compiles `module_damping_em.F` directly and
checks raw single-precision bits at 16 active and excluded points with non-one
memory origins. A full idealized Held-Suarez trajectory remains necessary in
addition to this routine-level coverage.

## WRF-006: repeated surface-pressure denominator

Status: source-confirmed performance opportunity; benchmark pending.

In both momentum loops in `dyn_em/module_damping_em.F:40-62`, the denominator
of `sig` uses pressure only at vertical index `1`. It therefore depends on the
two horizontal stagger points but not on loop variable `k`. The current loop
nesting is `j`, `k`, `i`, so the same four surface-pressure values are loaded,
added, and divided into every active vertical level.

Hoisting the denominator is not automatically a win. Reordering to `j`, `i`,
`k` sacrifices contiguous `i` traversal, while storing reciprocals adds scratch
traffic. A compiler may also eliminate part of the repeated work. Suggested
upstream investigation: inspect optimized code and benchmark the current loop
against a vector-friendly horizontal reciprocal buffer or a fused calling
context on representative domains. Preserve single-precision expression order
when evaluating numerical impact.

## WRF-007: column-mass staggering test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds `calc_mu_staggered`, its internal calls from
`couple`, and no dedicated numerical regression. The routine has distinct
interior, lower-boundary, upper-boundary, and both-boundaries paths for each
horizontal axis. A useful upstream test should cover all eight axis/path
combinations, subdomain tiles that rely on halos, exact single-precision
rounding, and untouched storage outside the active rectangles.

The local differential oracle now exercises interior, lower-boundary,
upper-boundary, and both-boundaries paths on both staggerings. It checks all
240 output and sentinel values by raw bits, including WRF's cross-axis domain
clipping. This closes the routine-level local gap while leaving the upstream
test gap unchanged. Periodic `calc_mu_uv` variants and full idealized-case
integration remain separate coverage requirements.

## WRF-008: avoidable intermediate overflow during normalization

Status: reproduced numerical robustness limitation outside ordinary atmospheric
input ranges; not classified as an operational forecast bug.

Both positive-definite routines normalize translated values with left-to-right
single-precision multiplication:

```fortran
line = line*f_total*rftotal_post
```

For the slab, `f_total` is the original line total; the sheet uses the supplied
target total. A translated finite value near `2e30` multiplied by a finite
target near `1e20` overflows before multiplication by the small reciprocal,
even though multiplying by the combined scale factor would produce a finite
normalized result. The seeded sheet case `1695930` and slab case `2771003`
reproduce positive infinities from finite inputs with the pinned routine. Rust
retains these bits for compatibility.

Suggested upstream investigation: establish realistic field bounds first, then
consider a scale-safe normalization such as computing the ratio before applying
it. Any change must evaluate underflow and rounding, because reassociation will
alter ordinary single-precision results. At minimum, a focused regression could
document the accepted behavior for extreme finite inputs.

## WRF-009: invalid `sprintf` arguments in empty-directory generator paths

Status: confirmed latent C bug; the normal Registry executable passes `inc`
and does not take the faulty branches.

Clang diagnoses `tools/gen_allocs.c:62` and `:843` while building the pinned
Registry tool. Both locations contain the same empty-directory branch:

```c
sprintf(fname,"%s%d.F",dirname,filename_prefix,idx ) ;
```

The format has two conversions, `%s` and `%d`, but receives three data
arguments. `filename_prefix` is a `char *` supplied to `%d`, and `idx` is then
unused. Variadic format mismatch is undefined behavior and can produce an
invalid filename or crash when `dirname` is empty. The non-empty branch uses
the intended format:

```c
sprintf(fname,"%s/%s%d.F",dirname,filename_prefix,idx) ;
```

Suggested upstream fix: change both empty-directory branches to
`sprintf(fname, "%s%d.F", filename_prefix, idx)`, preferably replacing all
unbounded filename formatting in these functions with checked `snprintf`.
Add a generator test that calls allocation and deallocation generation with
both empty and non-empty output directories.

## WRF-010: `task_for_point` assumes one-based domain origins

Status: reproduced correctness bug outside WRF's normal one-based coarse-domain
decomposition; two adjacent undefined-behavior defects are also compiler-confirmed.

`external/RSL_LITE/task_for_point.c` converts inclusive inputs to zero-based
indices, but the final centered-remainder branch mixes absolute indices with
the offset-only values `a` and `b`. In both axes, expressions such as
`(b-a-ids)` and `(i-b-ids)` subtract a nonzero domain origin from an already
relative boundary. Direct calls with `ids != 1` or `jds != 1` therefore assign
points to the wrong process or leave later process rows empty. The local
differential fixture confirmed normal `ids = jds = 1` behavior and keeps
offset-origin coverage as a Rust-only robustness test.

The same function is declared to return `int` but reaches the closing brace
without returning a value. Its MIC/HOST error path also calls `sprintf` with
two `%d` conversions and no matching arguments:

```c
sprintf(tfpmess,"%d by %d decomp will not work for MIC/HOST splitting. Need even number of tasks\n") ;
```

Suggested upstream fix: express `a` and `b` consistently as offsets, remove
the extra origin subtraction in the final branch, return an explicit status,
and pass the intended process dimensions to bounded `snprintf`. Add direct
tests with positive and negative non-one origins, odd remainders, and the
MIC/HOST rejection path.

## WRF-011: Kessler ignores passed thermodynamic constants in part of the update

Status: source-confirmed numerical inconsistency, including with WRF's default
model constants; scientific intent requires maintainer confirmation.

`phys/module_mp_kessler.F:81` calculates the saturation-adjustment factor with
the passed `xlv` and `cp` values:

```fortran
f5 = svp2*(svpt0-svp3)*xlv/cp
```

The later latent-heating and pressure expressions at lines 215-216 instead use
hard-coded values:

```fortran
pressure = 1.000e+05 * (pii(i,k,j)**(1004./287.))
gam = 2.5e+06/(1004.*pii(i,k,j))
```

WRF's default `module_model_constants.F` defines `cp = 7*r_d/2 = 1004.5`, not
1004. Consequently, even the normal default call uses one heat capacity in
`f5` and another in `gam` and the pressure exponent. Alternate caller-provided
`xlv` or `cp` values affect only part of the update.

The Rust port intentionally preserves the mixed constants because all 660
oracle outputs require exact parity. Suggested upstream investigation: confirm
whether the legacy 1004 values are scientifically intentional. If not, replace
them with the passed constants (and pass `r_d` if required), then add a
regression using non-default constants. This change would alter results and
must be treated as a scientific behavior change rather than cleanup.

## WRF-012: Kessler sedimentation assumes `kts = 1`

Status: source-confirmed latent bounds and interface defect. Normal ARW calls
appear to use `kts = 1`; operational impact outside that contract is not yet
established.

The routine accepts `kts` and declares scratch arrays with bounds `kts:kte`,
but initialization and spacing loops at lines 110 and 120 start from literal
level `1`. Bottom density and precipitation also read literal level `1` at
lines 114 and 144. Later loops switch back to `kts`.

If `kts > 1`, accesses such as `vt(1)`, `prodk(1)`, and `rhok(1)` fall outside
the declared automatic-array bounds. If a memory allocation contains level 1
but the active tile begins higher, the routine also processes excluded levels
before writing only `kts:kte` back to `prod`.

The Rust region rejects non-surface vertical starts explicitly rather than
pretending the upstream routine supports them. Suggested upstream fix: either
document and assert `kts == 1`, or consistently use `kts` as the lower bound and
define which density level supplies the fall-speed correction. Add a
`-fcheck=bounds` regression with `kts > 1` before accepting the latter change.

## WRF-013: Kessler numerical test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the scheme definition, physics-driver calls,
and tangent-linear/adjoint variants, but no dedicated numerical regression for
`module_mp_kessler.F`. Important branches include dry and saturated columns,
the cloud autoconversion threshold, rain accretion, evaporation limits,
adaptive sedimentation substeps, precipitation accumulation, and untouched
horizontal halos.

The local differential oracle compiles the pinned module directly and compares
all 660 mutable outputs by raw single-precision bits. It covers four rain
regimes, both sides of the cloud threshold, multi-step fallout, non-one
horizontal origins, and halo sentinels. A future upstream test should also add
exceptional values and a coupled precipitation trajectory.
