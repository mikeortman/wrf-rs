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
| WRF-014 | Test gap | Source and build-rule inventory | NetCDF I/O | The bundled `testWRFWrite.F90` and `testWRFRead.F90` are not build targets and call obsolete unprefixed external entry points absent from the current I/O library |
| WRF-030 | Confirmed latent bug | Source-confirmed; Rust validation reproduces boundary | `calculate_full` | The loop unconditionally reads `its-1` and `jts-1` without checking those indices against memory bounds |
| WRF-031 | Confirmed interface defect | Source-confirmed | `calculate_full` | A partially written array is declared `INTENT(OUT)`, leaving inactive storage undefined by the Fortran standard |
| WRF-032 | Test gap | Repository search plus coupled differential fixture | `rk_step_prep` | No dedicated numerical regression checks the seven production diagnostics together or observes their intermediate fields |
| WRF-033 | Performance/API opportunity | Source-confirmed, not independently benchmarked | `rk_step_prep` | The wrapper accepts several arguments that none of its seven diagnostic calls read |
| WRF-034 | Interface defect/API opportunity | Source-confirmed | `rk_addtend_dry` | `mu_tendf` is declared `INTENT(INOUT)` but only read; three map arrays and all patch bounds are unused |
| WRF-035 | Test gap | Repository search plus direct differential fixture | `rk_addtend_dry` | No dedicated production numerical regression checks first/later substeps, distinct stagger bounds, persistent fields, or inactive storage |
| WRF-036 | Interface/performance opportunity | Source-confirmed | `small_step_prep` | Ten numerical dummy arguments and three lower domain bounds are not read; production full-column execution also performs boundary stores that are overwritten before return |
| WRF-037 | Confirmed interface defect | Source-confirmed | `small_step_prep` | Twelve partially written arrays are declared `INTENT(OUT)`, making their inactive storage undefined on entry even though callers and differential tests may expect preserved halos |
| WRF-038 | Test gap | Repository search plus direct differential fixture | `small_step_prep` | No focused production regression checks first/later time-level behavior, all staggered bounds, exceptional arithmetic, or inactive storage |
| WRF-039 | Interface/API opportunity | Source-confirmed | `calc_p_rho` | Six coefficient/coordinate arrays and three lower domain bounds are accepted but never read |
| WRF-040 | Confirmed interface defect | Source-confirmed | `calc_p_rho` | Partially written `al` and `p` arrays are declared `INTENT(OUT)`, making inactive tile and halo storage undefined on entry |
| WRF-041 | Test gap | Repository search plus direct differential fixture | `calc_p_rho` | No focused production regression jointly checks both governing modes, damping phases, hydrostatic recurrence, partial tiles, or exceptional arithmetic |

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

A repository-wide search finds `calc_mu_staggered`, `calc_mu_uv`, and
`calc_mu_uv_1`, their internal calls, and no dedicated numerical regression.
The routines have distinct interior, lower-boundary, upper-boundary, and
both-boundaries paths for each horizontal axis. The big-step variants also
select physical or periodic endpoint expressions independently on each axis. A
useful upstream test should cover those paths, subdomain tiles that rely on
halos, exact single-precision rounding, and untouched storage outside the active
rectangles.

The local differential oracle now exercises interior, lower-boundary,
upper-boundary, and both-boundaries paths on both staggerings. It checks all
240 output and sentinel values by raw bits, including WRF's cross-axis domain
clipping. The big-step oracle adds all four periodicity states for both
split-mass and already-combined-mass entry points, checks all 960
output/sentinel values, and includes a finite input that distinguishes WRF's
duplicate endpoint average from a direct copy. This closes the focused
routine-level local gap while leaving the upstream test gap unchanged. A
randomized big-step corpus and full idealized-case integration remain separate
coverage requirements.

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

## WRF-014: orphaned NetCDF read/write tests

Status: source-confirmed test infrastructure gap; no claim that production WRF
NetCDF output is broken.

`external/io_netcdf/README` presents `testWRFWrite.F90` and
`testWRFRead.F90` as the package's tests. The current directory makefile has no
target that compiles or runs either program; their names appear only in the
cleanup rule. The programs call unprefixed procedures such as
`ext_open_for_write_begin` and `ext_write_field`, while the current library
defines the `ext_ncd_*` entry points with expanded metadata and decomposition
arguments. A repository search finds no compatibility definitions for the
unprefixed procedures in `external/io_netcdf` or `external/ioapi_share`.

This leaves the low-level NetCDF backend without a maintained package-local
round-trip gate. Suggested upstream action: either remove the programs and
README instructions as historical artifacts, or update them to current
interfaces and add a build/CTest target that checks schema, field values,
multiple records, classic/NetCDF-4 modes, and restart metadata.

## WRF-015: unused `msfv` argument in `couple_momentum`

Status: source-confirmed interface redundancy; no numerical defect.

`couple_momentum` accepts both `msfv` and `msfv_inv` and declares both as input
arrays. The south-north equation reads only `msfv_inv`; a repository search of
the exact routine finds no reference to `msfv` after its declaration. Current
ARW callers nevertheless pass both `msfvx` and `msfvx_inv`.

The Rust capability carries only the inverse factor actually used by the
algorithm. Suggested upstream action: remove `msfv` in an interface-breaking
cleanup, or at minimum comment that it is retained for compatibility. Compiler
warnings for unused dummy arguments would make similar drift easier to find.

## WRF-016: momentum-coupling numerical test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the production `couple_momentum` routine,
Runge-Kutta preparation call, and tangent-linear/adjoint derivatives, but no
dedicated numerical regression for the production routine. Its three loops use
different horizontal and vertical clipping, different map-factor operations,
and half- versus full-level coefficients.

The local differential oracle extracts the exact production body and checks all
3,780 output and inactive sentinel values by raw single-precision bits. It
covers each upper stagger independently and together, negative/non-one memory
origins, finite overflow, division by zero, and multiplication by a zero inverse
factor. A useful upstream regression should add the same branch geometry and
then exercise the operation through `rk_step_prep` in an idealized trajectory.

## WRF-017: four unused map-factor arguments in `calc_ww_cp`

Status: source-confirmed interface redundancy; no numerical defect.

`calc_ww_cp` accepts and declares `msfty`, `msfux`, `msfvx`, and `msfvy` at
`dyn_em/module_big_step_utilities_em.F:640-667`. The executable statements at
lines 692-779 read `msftx`, `msfuy`, and `msfvx_inv`, but never reference the
other four arrays. The sole production caller still passes all seven factors.

The Rust capability carries only the three arrays read by the algorithm.
Suggested upstream action: remove the four unused arguments in an
interface-breaking cleanup, or document that they are retained for call-site
compatibility. Enabling unused-dummy-argument warnings would identify similar
signature drift.

## WRF-018: `calc_ww_cp` has an implicit complete-column precondition

Status: source-confirmed latent bounds/initialization defect. Normal ARW calls
appear to provide the complete vertical column; impact outside that path is not
established.

The routine accepts `kts` and declares `divv(its:ite,kts:kte)`, but line 714
writes `ww(i,1,j)` and the recurrence at line 767 starts from literal level
`2`. If `kts > 1`, the recurrence reads `divv(i,k-1)` below its declared lower
bound and uses output levels excluded from the tile. If `kte < kde`, the zero
written at `kte` can be overwritten because `ktf = min(kte,kde-1)`.

The derivation in the routine itself integrates over all levels and assumes
zero vertical flux at the physical top and bottom, so a partial vertical tile
is not merely an indexing variation. The Rust region rejects incomplete
columns explicitly. Suggested upstream fix: document and assert
`kts == kds == 1` and `kte == kde`, or redesign the interface and mathematics
for partial columns. Add a bounds-checked regression for invalid `kts`/`kte`.

## WRF-019: `calc_ww_cp` partially defines an `INTENT(OUT)` array

Status: source-confirmed Fortran interface defect; normal callers may not read
inactive storage.

`ww` is declared `INTENT(OUT)` at line 666. Fortran makes the whole dummy array
undefined on entry, but the routine assigns only active horizontal points,
bottom/top tile points, and internal active full levels. Horizontal halos,
clipped rows, vertical storage outside the physical column, and most upper
west-east stagger points are not assigned.

Observed compilers leave those bytes unchanged, which is why the local oracle
can verify sentinel preservation, but relying on their prior values is not
standard-conforming. Suggested upstream action: change the declaration to
`INTENT(INOUT)` if preservation is intended, or assign the complete declared
array and document the value for inactive storage.

## WRF-020: omega-diagnosis numerical test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the production `calc_ww_cp` routine,
`rk_step_prep` call, and tangent-linear/adjoint variants, but no dedicated
numerical regression. The routine combines staggered column-mass averaging,
component-specific map factors, horizontal divergence, a vertical reduction,
and a recurrence with fixed physical-boundary values.

The local differential oracle extracts the exact production body and compares
all 1,960 stored output and sentinel values. It covers interior and upper-edge
tiles, combined boundaries, non-one/negative memory origins, finite overflow,
zero map factors, and untouched storage. A useful upstream regression should
add the same geometry, a full-column contract check, randomized finite inputs,
and an integrated `rk_step_prep` trajectory.

## WRF-021: `calc_cq` assumes west and south halo neighbors

Status: source-confirmed latent bounds precondition. Normal ARW decomposition
provides these halos; impact is limited to callers that violate the implicit
tile contract.

The `cqu` loop reads `moist(i-1,k,j,ispe)` beginning at `i=its`, and the `cqv`
loop reads `moist(i,k,j-1,ispe)` beginning at `j=jts`. The routine accepts
memory and tile bounds independently but does not document or check that
`its > ims` and `jts > jms`. A tile beginning at its allocation lower bound
therefore reads outside the declared dummy-array bounds when active species
exist.

The Rust region rejects missing west and south neighbors before mutation.
Suggested upstream action: document and assert the halo precondition, or pass
validated domain descriptors rather than independent integers. Add a
`-fcheck=bounds` regression with each missing neighbor.

## WRF-022: `calc_cq` partially defines three `INTENT(OUT)` arrays

Status: source-confirmed Fortran interface defect; normal callers may not read
inactive storage.

`cqu`, `cqv`, and `cqw` are all declared `INTENT(OUT)`, which makes each entire
dummy array undefined on entry. The routine assigns only component-specific
tile ranges: `cqu` retains the upper west-east stagger, `cqv` retains the upper
south-north stagger, and `cqw` excludes the tile's first vertical level. Halos,
clipped upper points, and other allocated levels are not assigned.

Observed GNU Fortran builds leave inactive bytes unchanged, enabling the local
oracle to check sentinels, but prior values are not standard-conforming after
the call. Suggested upstream action: use `INTENT(INOUT)` when preservation is
the contract, or initialize every declared element and document its dry value.

## WRF-023: `calc_cq` carries avoidable row scratch

Status: source-confirmed performance opportunity; model-level impact has not
been measured.

The routine declares automatic `qtot(its:ite)` storage, clears the complete row
for every active `(j,k)` pair, accumulates species into it, then makes another
pass to write the output. For `cqv` and `cqw`, `itf` may be clipped below `ite`,
so `qtot = 0.` also clears values that cannot be read. Modern compilers may
optimize some traffic, and this finding does not claim a measured WRF defect.

The parity-equivalent Rust implementation uses each active output row as the
temporary total and overwrites it with the final coefficient, removing
numerical scratch while preserving per-point addition order. Suggested
upstream experiment: accumulate directly into the corresponding output range,
restrict initialization to `its:itf`, and benchmark with exact-output checks.

## WRF-024: moisture-coefficient numerical test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the production `calc_cq` routine, its
`rk_step_prep` call, downstream uses, and tangent-linear/adjoint variants, but
no dedicated numerical regression for the production routine. Its zero-species
branch, generated first-scalar index, three distinct staggers, lower-neighbor
reads, and component-specific clipping are otherwise easy to regress.

The local differential oracle extracts the exact production body and compares
all 8,232 output and sentinel values across zero, one, and three active species,
upper boundaries, non-one origins, vertical clipping, signed zero, and finite
overflow. A useful upstream test should add the same geometry and then exercise
the coefficients through `rk_step_prep` and a small-step trajectory.

## WRF-025: `calc_alt` partially defines an `INTENT(OUT)` array

Status: source-confirmed Fortran interface defect; normal callers may not read
inactive storage.

`alt` is declared `INTENT(OUT)` at line 924, which makes the complete dummy
array undefined on entry. The routine clips all three upper tile endpoints at
lines 936–938 and writes only the resulting active mass points at line 943.
Halos, upper stagger storage, and points outside a subdomain tile are not
assigned.

Observed GNU Fortran builds retain prior bytes in inactive storage, enabling
sentinel verification, but those values are not standard-conforming after the
call. Suggested upstream action: use `INTENT(INOUT)` if preservation is the
contract, or assign the complete declared array and document inactive values.

## WRF-026: full inverse-density numerical test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the production `calc_alt`, its `rk_step_prep`
call, initialization equivalents, and many downstream `alt` consumers, but no
dedicated numerical regression for the production routine. Although the
arithmetic is one addition, independent tile/domain/memory bounds and partial
`INTENT(OUT)` writes make boundary regressions observable.

The local differential oracle extracts the exact routine body and compares all
2,352 stored output and sentinel values across six cases. It covers each upper
axis independently and together, negative/non-one memory origins, finite
overflow, cancellation, signed zero, opposite infinities, untouched storage,
and worker-count determinism. A useful upstream test should add the same
geometry and exercise `alt` through an integrated `rk_step_prep` trajectory.

## WRF-027: `calc_php` assumes a stored upper full level

Status: source-confirmed latent bounds precondition. Normal ARW grids provide
the required vertical stagger; impact is limited to invalid independent bounds.

The loop clips `ktf` to `kde-1` at line 1256, then reads both `ph(i,k+1,j)` and
`phb(i,k+1,j)` at line 1261. The routine accepts `kde` and `kme` independently
but does not document or check that the memory allocation includes logical
level `kde`. A caller with `kde > kme` can therefore read outside the declared
dummy-array bounds.

The Rust region rejects a missing upper full level before mutation. Suggested
upstream action: document and assert `kde <= kme`, or replace independent
integers with a validated grid descriptor. Add a `-fcheck=bounds` regression
for the missing-neighbor case.

## WRF-028: `calc_php` partially defines an `INTENT(OUT)` array

Status: source-confirmed Fortran interface defect; normal callers may not read
inactive storage.

`php` is declared `INTENT(OUT)` at line 1241, making the complete dummy array
undefined on entry. The routine clips all upper tile endpoints at lines
1254–1256 and writes only active mass points at line 1261. Halos, upper stagger
storage, and points outside a subdomain tile are not assigned.

Observed GNU Fortran builds retain prior bytes in inactive storage, enabling
sentinel verification, but those values are not standard-conforming after the
call. Suggested upstream action: change `php` to `INTENT(INOUT)` when
preservation is intended, or assign the complete array and document inactive
values.

## WRF-029: pressure-point geopotential numerical test coverage

Status: confirmed repository-level test gap for the pinned source tree.

A repository-wide search finds the production `calc_php`, its `rk_step_prep`
call, equivalent initialization expressions, and downstream pressure-gradient
consumers, but no dedicated numerical regression for the production routine.
Its apparent simplicity hides three-axis clipping, a vertical neighbor, partial
output definition, and floating-point operation-order sensitivity.

The local differential oracle extracts the exact routine body and compares all
2,352 stored output and sentinel values across six cases. It covers independent
and combined upper boundaries, negative/non-one memory origins, the required
upper full level, source-order-sensitive overflow, signed zero, opposite
infinities, untouched storage, and worker-count determinism. A useful upstream
test should add the same geometry and then exercise all seven diagnostics in an
integrated `rk_step_prep` trajectory.

## WRF-030: `calculate_full` assumes two lower halo points

Status: source-confirmed latent bounds precondition. Normal decomposed ARW
patches communicate the required halos, but the routine accepts independent
memory and tile bounds without checking the relationship.

At lines 3618–3619, `calculate_full` sets `i_start=its-1` and
`j_start=jts-1`. Its loop then reads and writes those indices. A caller with
`its <= ims` or `jts <= jms` accesses outside the corresponding declared dummy
array even though every supplied bound is individually valid.

The Rust full-column-mass stage rejects either missing lower halo before
mutation. Suggested upstream action: document and assert `its > ims` and
`jts > jms`, or pass a validated bounds descriptor. Add a `-fcheck=bounds`
case with the tile beginning at the memory lower bound.

## WRF-031: `calculate_full` partially defines an `INTENT(OUT)` array

Status: source-confirmed Fortran interface defect.

`rfield` is declared `INTENT(OUT)` at line 3601, making its complete dummy
array undefined on entry. The loop writes only `its-1:MIN(ite,ide-1)`, one
vertical tile range, and `jts-1:MIN(jte,jde-1)`. Storage outside that rectangle
is not assigned. GNU Fortran happens to retain previous inactive bytes, but a
caller may not portably read them after return.

Suggested upstream action: use `INTENT(INOUT)` if halo and off-tile preservation
is the contract, or define every element. The coupled local oracle verifies all
active values and observes sentinels in the current GNU build without treating
that observation as a language guarantee.

## WRF-032: no coupled numerical regression for `rk_step_prep`

Status: confirmed repository-level test gap for the pinned source tree.

The production wrapper calls seven diagnostics whose intermediate fields feed
later stages, but a repository-wide search finds no dedicated regression that
runs this sequence and compares `mut`, `muu`, `muv`, `ru`, `rv`, `rw`, `ww`,
`cqu`, `cqv`, `cqw`, `alt`, and `php` together. Per-routine tests would still
miss argument wiring and ordering errors at the wrapper boundary.

The local coupled oracle extracts all seven exact routine bodies, executes them
in production order, and compares every one of 1,728 stored values by raw bits.
Suggested upstream action: add a small `rk_step_prep` fixture that observes both
intermediate mass fields and final diagnostics, then extend it into an
idealized short trajectory.

## WRF-033: `rk_step_prep` carries dead wrapper arguments

Status: source-confirmed API and maintenance opportunity; no independent speed
claim is made.

Within `rk_step_prep`, `rk_step`, `t`, `pb`, `p`, `fnm`, and `fnp` are not read
or forwarded to any of the seven calls. The wrapper also forwards map-factor
arrays already recorded as unused by `calc_ww_cp` in WRF-017. These arguments
increase an already large positional interface and make call-site wiring harder
to review.

The Rust integration boundary retains only participating data and groups it by
scientific role. Suggested upstream action: remove dead arguments in a planned
interface change, or introduce a derived state/configuration object and enable
unused-dummy-argument warnings to prevent further drift.

## WRF-034: `rk_addtend_dry` has misleading and dead dummy arguments

Status: source-confirmed interface defect and API maintenance opportunity.

`mu_tendf` is declared `INTENT(INOUT)` but is only read by
`mu_tend = mu_tend + mu_tendf`. The routine also accepts `msftx`, `msfux`, and
`msfvy` without reading them. All six patch bounds are unused, as are the lower
domain bounds and vertical domain bounds in the executable body. Memory bounds
remain necessary for the explicit-shape dummy arrays.

The mutable declaration overstates the routine's write set and prevents a
compiler or caller from expressing the actual read-only contract. The Rust API
borrows forward column-mass tendency immutably and omits unused map factors and
patch bounds. Suggested upstream action: change `mu_tendf` to `INTENT(IN)` and
remove dead arguments in a planned interface revision, or enable unused-dummy
warnings to keep the positional interface from drifting further.

## WRF-035: no focused numerical regression for `rk_addtend_dry`

Status: confirmed repository-level test gap for the pinned source tree.

A repository search finds production, tangent-linear, and adjoint variants but
no dedicated numerical regression for the production routine. This leaves the
first-substep mutation of persistent tendencies, later-substep reuse, diabatic
heating, all twelve observable mutable fields, and inactive storage unchecked.
The stagger bounds are particularly easy to mis-port: U and V have different
horizontal upper points, while W/geopotential use `kts:kte` and temperature and
horizontal momentum use `kts:kte-1`. Full-domain clipping can hide the latter
difference; an interior-tile oracle exposed it during this port.

The local fixture extracts the exact pinned body and compares 5,616 complete
values across first, later, and exceptional cases. Suggested upstream action:
add a compact fixture covering both RK phases, an interior vertical tile, all
upper staggers, persistent fields, and untouched halos, then include it in a
short prognostic trajectory.

## WRF-036: `small_step_prep` carries dead arguments and overwritten stores

Status: source-confirmed interface and performance opportunity; no independent
whole-model speed claim is made.

The executable body never reads `c3h`, `c4h`, `c3f`, `c4f`, `msfux`, `msfvx`,
`msfvy`, `msftx`, `rdx`, or `rdy`. The lower domain bounds `ids`, `jds`, and
`kds` are also unused. Memory bounds remain necessary for the explicit-shape
dummy arrays, while upper domain bounds participate in clipping.

The routine stores zero into `ww_save(i,kde,j)` and `ww_save(i,1,j)` at lines
133–134 and again at 225–226. Its production full-column path subsequently
copies `ww(i,k,j)` for every `k=1:kde` at lines 281–287, so none of those zero
stores is observable. The Rust routine requires the same full-column contract
and omits only these overwritten stores; its complete output oracle remains
exact.

Suggested upstream action: remove dead dummy arguments during a planned
interface revision, enable unused-dummy warnings, and remove the boundary zero
stores if full-column execution is an explicit invariant. If partial vertical
tiles are meant to be supported, document and test that different contract
before changing the stores.

## WRF-037: `small_step_prep` partially defines `INTENT(OUT)` arrays

Status: source-confirmed Fortran interface defect.

`u_save`, `v_save`, `w_save`, `t_save`, `ph_save`, `c2a`, `ww_save`, `muus`,
`muvs`, `muts`, `mudf`, and `mu_save` are declared `INTENT(OUT)` at lines
52–65 and 86–91. The routine writes only each active tile and its
component-specific stagger or vertical range. Under the Fortran standard,
association with an `INTENT(OUT)` dummy makes the complete actual argument
undefined on entry; inactive halo and off-tile storage therefore cannot
portably retain its prior value.

GNU Fortran currently leaves those inactive bytes unchanged, which permits the
local oracle to observe sentinels, but that is not a language guarantee. The
Rust API models these fields as mutable caller-owned storage and deliberately
preserves inactive values.

Suggested upstream action: use `INTENT(INOUT)` if halo/off-tile preservation is
the contract, or explicitly initialize the complete arrays before returning.
Add a regression that checks both active values and the chosen inactive-storage
policy.

## WRF-038: no focused numerical regression for `small_step_prep`

Status: confirmed repository-level test gap for the pinned source tree.

A repository search finds the production calls but no dedicated numerical
fixture for this routine. The missing coverage includes first-substep time-level
replacement, later-substep differences, four distinct stagger ranges,
west/south neighbor averages, the complete W/geopotential column, saved state,
pressure-coefficient arithmetic, exceptional IEEE behavior, and inactive
storage.

The local fixture extracts the exact pinned body and compares all 9,936 mutable
and sentinel values across first, later-interior, and exceptional cases.
Suggested upstream action: adopt a similarly small routine-level fixture, then
extend it into an acoustic trajectory that observes state after every small
step.

## WRF-039: `calc_p_rho` carries nine dead dummy arguments

Status: source-confirmed API maintenance opportunity; no independent
whole-model speed claim is made.

The executable body never reads full-level coefficient arrays `c1f`, `c2f`,
`c3f`, and `c4f`, half-level array `c4h`, or vertical coordinate `znu`. Lower
domain bounds `ids`, `jds`, and `kds` are also unused. The memory bounds remain
necessary for explicit-shape dummy arrays, while upper domain and tile bounds
participate in clipping.

These arguments make a large positional call harder to audit and imply
dependencies that do not exist. The Rust capability keeps only `c1h`, `c2h`,
`c3h`, `rdnw`, and `dnw`, grouped by scientific role. Suggested upstream
action: remove dead dummies during a planned interface revision or enable
unused-dummy warnings to prevent further drift.

## WRF-040: `calc_p_rho` partially defines `INTENT(OUT)` arrays

Status: source-confirmed Fortran interface defect.

`al` and `p` are declared `INTENT(OUT)` at lines 460–461, but the routine only
writes `its:min(ite,ide-1)`, `jts:min(jte,jde-1)`, and
`kts:min(kte,kde-1)`. Under the Fortran standard, association with an
`INTENT(OUT)` dummy makes the complete actual argument undefined on entry, so
inactive tile and halo storage cannot portably retain its prior value.

GNU Fortran currently preserves those inactive bytes, enabling the local
fixture to observe sentinels, but this is not a language guarantee. Rust uses
mutable caller-owned storage and explicitly leaves inactive values unchanged.
Suggested upstream action: use `INTENT(INOUT)` if preservation is the intended
contract, or define the complete arrays and test that policy.

## WRF-041: no focused numerical regression for `calc_p_rho`

Status: confirmed repository-level test gap for the pinned source tree.

The routine is called before and during every acoustic loop, yet a repository
search finds no dedicated production numerical fixture. Missing branch
coverage includes nonhydrostatic geopotential-gradient diagnosis, hydrostatic
pressure and upward geopotential recurrence, initialization versus forward
pressure damping, partial vertical tiles, inactive storage, and zero/infinite
denominators.

The local oracle extracts the exact pinned routine and compares all 3,456
stored values across both governing modes and both damping phases, including
partial and exceptional cases. Suggested upstream action: adopt a compact
routine fixture and include pressure, inverse density, geopotential, and
pressure history in a per-substep acoustic trajectory comparison.

## WRF-042: `calc_coef_w` carries dead arguments and misleading vertical tile plumbing

Status: source-confirmed interface maintenance opportunity.

The executable body never reads `c3h`, `c4h`, `c3f`, or `c4f`. Lower domain
bounds `ids`, `jds`, and `kds` are also unused. The routine assigns `kts` and
`kte-1` to local `k_start` and `k_end`, but never reads either local afterward;
it always constructs the complete column using literal lower indices and
`kde`. Locals `ij`, `ijp`, and `ijm` are declared but unused as well.

The full-column behavior is consistent with the implicit vertical solve, but
the accepted vertical tile arguments imply a capability that does not exist.
The Rust region omits a vertical tile and requires the complete half-level
domain plus the top full level. Suggested upstream action: remove dead
arguments and locals during a planned interface revision, or document and
assert the complete-column invariant at the call boundary.

## WRF-043: `calc_coef_w` allocates an unnecessary horizontal `cof` array

Status: source-confirmed local memory and clarity opportunity; no independent
whole-model speed claim is made.

`cof(i)` is assigned the same expression,
`(.5*dts*g*(1.+epssm))**2`, for every active west–east point. The value has no
`i` dependency, so the automatic `ims:ime` array and initialization loop are
unnecessary. The Rust implementation calculates one scalar per call and
preserves exact output bits.

Suggested upstream action: replace `cof(:)` with a scalar calculated once
outside the horizontal loops. This removes stack storage and makes the actual
dependency explicit.

## WRF-044: rigid-lid multiplication does not guarantee a zero exceptional coefficient

Status: source-confirmed exceptional-value robustness issue.

When `top_lid` is true, `lid_flag` becomes zero and is multiplied into the top
lower-diagonal numerator only after inverse spacing and pressure terms have
already been evaluated. Consequently, an infinite intermediate produces
`0 * Inf = NaN`, and a zero denominator can produce `0 / 0 = NaN`, instead of
the mathematically intended zero rigid-lid coupling. The local oracle confirms
this IEEE behavior, and Rust preserves it for parity.

Suggested upstream action: if rigid-lid decoupling must hold even after an
upstream numerical failure, branch explicitly and assign `a(i,kde,j)=0.`.
Changing this behavior should be accompanied by a documented exceptional-value
policy because it alters current observable results.

## WRF-045: no focused numerical regression for `calc_coef_w`

Status: confirmed repository-level test gap for the pinned source tree.

A repository search finds production use of the factors but no dedicated
numerical fixture for lower/interior/top coefficients, rigid versus nonrigid
upper boundaries, forward elimination, partial horizontal tiles, complete
vertical columns, inactive storage, or exceptional denominators.

The local fixture extracts the exact pinned routine and compares all 3,024
`a`, `alpha`, `gamma`, and sentinel values across four cases. Suggested
upstream action: adopt a compact coefficient fixture and then validate the
factorization through the subsequent forward and backward `advance_w` sweeps.

## WRF-046: partial nonhydrostatic `advance_uv` can read uninitialized `dpn`

Status: source-confirmed correctness defect.

The routine initializes `dpn(:,1)` and `dpn(:,kde)`, then fills only
`k_start+1:k_end`. Its pressure-gradient loop reads `dpn(:,k_start)`. When a
vertical tile begins above level 1, that value was never defined and the result
depends on automatic-array stack contents. Suggested upstream action: fill
`dpn(:,k_start)` with the ordinary `fnm/fnp` interpolation when `k_start > 1`
and add a partial-tile regression.

## WRF-047: `advance_uv` carries dead coefficient and bound plumbing

Status: source-confirmed interface maintenance opportunity.

`c1f`, `c2f`, `c3h`, `c4h`, `c3f`, and `c4f` occur only in the dummy interface.
`kds` is not read, while `k_endw` is assigned in both bound branches but never
consumed. Suggested upstream action: remove these during an interface revision
or enable unused-dummy/local diagnostics in focused builds.

## WRF-048: `advance_uv` tile scratch is avoidable and lacks a focused test

Status: confirmed test gap and local-memory opportunity.

Automatic `dpn`, `dpxy`, and `mudf_xy` arrays do not escape and their values
can be computed directly in the final U/V update. WRF's test tree has no
dedicated fixture spanning governing mode, top lid, relaxation, open,
symmetric, periodic, and polar branches. The safe Rust kernel uses no numerical
scratch and checks raw bits against the extracted routine. Suggested upstream
action: add a branch fixture before changing scratch lifetime or loop fusion.

## WRF-049: `advance_mu_t` carries fourteen dead arguments and two dead locals

Status: source-confirmed interface maintenance opportunity.

The executable body never reads `c1f`, `c2f`, `c3h`, `c4h`, `c3f`, `c4f`,
`msfux`, `msfvx`, or `msfvy`. It never reads or writes the declared `INOUT`
arrays `uam`, `vam`, and `wwam`, and it does not read `step` or `kds`.
Locals `i_endu` and `j_endv` are assigned but never consumed. The Rust
capability omits all of them. Suggested upstream action: remove dead plumbing
during an interface revision and enable unused-dummy/local warnings.

## WRF-050: `advance_mu_t` partially defines three `INTENT(OUT)` arrays

Status: source-confirmed Fortran interface defect.

`muave`, `muts`, and `mudf` are declared `INTENT(OUT)` but only active tile
points are assigned. Fortran makes the complete actual arrays undefined on
entry, so inactive halo/tile values cannot portably retain their contents even
though GNU Fortran currently leaves their bytes intact. Suggested upstream
action: use `INTENT(INOUT)` when inactive preservation is intended, or define
the complete arrays and test that policy.

## WRF-051: `advance_mu_t` accepts partial vertical tiles but requires a complete column

Status: source-confirmed correctness and contract defect.

The divergence loop honors `k_start`, but omega and theta loops restart from
literal levels 1 or 2. They then read `dvdxi` and `wdtn` values whose
initialization depends on a complete column ending at `kde`. A partial vertical
tile can therefore read uninitialized automatic-array elements. The Rust region
requires the complete half-level column plus its upper full level. Suggested
upstream action: document/assert that invariant or make every loop and boundary
initialization consistently honor the supplied tile.

## WRF-052: `advance_mu_t` scratch arrays are avoidable and lack focused regression

Status: confirmed local-memory opportunity and repository-level test gap.

The routine allocates tile-sized `wdtn` and `dvdxi` arrays plus horizontal
`dmdt`. Rust stores divergence briefly in the required `t_ave` output and prior
mass in `muts`, computes vertical theta transport locally, then writes the
specified final values. This removes numerical scratch while matching all
3,168 oracle values. WRF has no dedicated fixture spanning global, nested,
periodic-X, partial horizontal, complete-column, and inactive-storage behavior.
Suggested upstream action: add that fixture before refactoring scratch or loops.

## WRF-053: `advance_w` carries seven dead arguments and four dead locals

Status: source-confirmed interface maintenance opportunity.

The executable body never reads `mu1`, `c3h`, `c4h`, `c3f`, `c4f`, `dnw`, or
`alb`. Local array `mut_inv` and scalars `muthk` and `muthkm1` are declared but
never used. `k_start` is assigned from `kts` but never read. The Rust capability
omits these roles. Suggested upstream action: remove the dead plumbing during a
planned interface revision and enable unused-dummy/local diagnostics in a
focused build.

## WRF-054: `advance_w` accepts vertical tiles but requires a complete column

Status: source-confirmed correctness and contract defect.

The routine declares automatic `rhs` and `wdwn` with `kts:kte` bounds, but its
executable loops use literal level 1, level 2, `kte-1`, and the upper `kte`
point. The assigned `k_start = kts` is ignored. A tile beginning above the
surface can write outside those automatic arrays, and a tile ending below the
physical top cannot represent the required forward/back recurrence. The lower
boundary also unconditionally reads the first three velocity levels.

The Rust region requires the complete mass column, its upper full level, and
at least three mass levels. Suggested upstream action: document and assert the
complete-column contract at the call boundary, or consistently reformulate
every boundary and recurrence around supplied vertical bounds.

## WRF-055: `advance_w` uses avoidable automatic scratch around one required RHS

Status: source-confirmed local-memory opportunity; no independent whole-model
speed claim is made.

The geopotential `rhs` must coexist with old vertical momentum and
geopotential, so one complete tile workspace is justified. The additional
`wdwn` tile array can be evaluated from adjacent levels at its two consumption
sites. `msft_inv` and `dampwt` are arrays even though each value is consumed as
a scalar within its current row or level iteration. Rust retains one explicit,
caller-owned RHS field and evaluates the other quantities locally, matching all
2,592 state bits.

Suggested upstream action: after adding a direct regression, evaluate removing
the three auxiliary arrays and measure stack use plus end-to-end acoustic-step
performance before accepting the change.

## WRF-056: `advance_w` contains unresolved map-factor and surface-condition warnings

Status: source-commented scientific-review item, not yet proven as an output
defect.

The routine header says the use of `msfty` as an inverse map factor was not
fully worked through and describes a suspected surface vertical-velocity
error. Those warnings remain in v4.7.1 while the routine sets the terrain-
following lower boundary used by every nonhydrostatic acoustic step.

Suggested upstream action: resolve the comments against the governing mapped-
coordinate equations, document the intended discrete map-factor convention,
and add an analytic sloping-terrain fixture before changing current behavior.
Rust preserves the pinned result until that scientific decision is made.

## WRF-057: no focused numerical regression covers `advance_w`

Status: confirmed repository-level test gap for the pinned source tree.

A repository search finds production, tangent-linear, and adjoint call sites
but no dedicated nonlinear numerical fixture spanning both `phi_adv_z`
formulations, rigid and nonrigid tops, `damp_opt = 3`, terrain reconstruction,
specified/nested clipping, periodic X, partial horizontal tiles, inactive
storage, and the forward/back tridiagonal sweeps.

The local oracle extracts the exact pinned nonlinear routine and compares all
2,592 stored `w`, `ph`, and `t_2ave` values across four cases. Suggested
upstream action: adopt a compact routine fixture, then add a per-substep
trajectory containing `calc_coef_w`, `advance_uv`, `advance_mu_t`, and
`advance_w` together.

## WRF-058: `sumflux` carries ten dead arguments

Status: source-confirmed interface maintenance opportunity.

The executable routine never reads `c1f`, `c2f`, `c3h`, `c4h`, `c3f`, `c4f`,
`epssm`, `msfux`, `msfvx`, or `msfvy`. The Rust capability exposes only the
two coefficients and two map factors that affect output. Suggested upstream
action: remove the dead plumbing during a planned interface revision and
enable unused-dummy diagnostics for this focused source file.

## WRF-059: no focused regression covers `sumflux` stagger clearing and finalization

Status: confirmed repository-level test gap for the pinned source tree.

The first iteration clears the entire tile for all three outputs, while later
accumulation and finalization use distinct U, V, and W stagger ranges. A test
that inspects only active mass points misses observable zero stores on the
other upper stagger locations. The local oracle executes three substeps and
compares all 375 stored values, including halos and stagger-only points.

Suggested upstream action: add a compact complete-storage regression before
changing loop fusion or initialization behavior, then include `sumflux` in a
coupled acoustic trajectory fixture.

## WRF-060: no coupled numerical regression covers a complete local acoustic sequence

Status: confirmed repository-level test gap for the pinned source tree.

The seven nonlinear routines used by one local nonhydrostatic acoustic
trajectory are exercised through production drivers, but the repository has no
focused fixture that runs their exact `solve_em.F` order and compares the
prognostic state after multiple substeps. Routine-only tests cannot detect
cross-stage argument swaps such as confusing `fnm` and `fnp`, stale pressure
diagnostics, or an incorrectly placed flux finalization.

The local oracle extracts all seven exact routine bodies, executes preparation,
initial pressure and coefficient diagnosis, then three complete substeps, and
compares 2,196 selected final values by raw IEEE bits. Suggested upstream
action: add a small serial fixture with intermediate checkpoints, then reuse it
around halo and physical-boundary insertion in a multi-patch regression.
