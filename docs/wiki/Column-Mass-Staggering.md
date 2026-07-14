# Column-mass staggering

WRF's ARW dynamical core stores dry-air column mass at scalar, or mass-grid,
points. Horizontal momentum components live on an Arakawa C grid: west-east
momentum lies between adjacent mass points in the west-east direction, while
south-north momentum lies between adjacent rows. `calc_mu_staggered`,
`calc_mu_uv`, and `calc_mu_uv_1` construct the full dry-air mass factors used at
those two momentum locations in different ARW calling contexts.

## Interior algorithm

Let `mu` be perturbation dry-air column mass and `mub` the hydrostatic
base-state contribution. At an interior west-east momentum point, WRF computes

```text
muu(i,j) = 0.5 * (mu(i,j) + mu(i-1,j) + mub(i,j) + mub(i-1,j))
```

The south-north result uses the same arithmetic with row `j-1`:

```text
muv(i,j) = 0.5 * (mu(i,j) + mu(i,j-1) + mub(i,j) + mub(i,j-1))
```

The Rust implementation retains WRF's single precision and source operation
order. It does not first materialize `mu + mub`, because the different rounding
sequence can change output bits.

## Physical boundaries

A physical domain boundary is not a halo boundary. An interior subdomain tile
has valid preceding mass values in its memory halo and uses the average above.
A tile touching the global lower or upper boundary must instead copy the
nearest full mass; there is no mass point beyond the physical edge.

| Axis path | Lower momentum point | Interior momentum points | Upper momentum point |
|---|---|---|---|
| Interior | average | average | average |
| Lower only | `mu + mub` at the current mass point | average | average |
| Upper only | average | average | `mu + mub` at the preceding mass point |
| Both | current-point copy | average | preceding-point copy |

WRF evaluates these four paths independently for west-east and south-north
momentum. The Rust port derives the same state for each axis and does not ask a
caller to supply boolean boundary flags.

## Big-step and periodic variants

`calc_mu_uv` accepts perturbation and base mass separately. `calc_mu_uv_1`
accepts an already-combined full-mass field. Rust exposes both operations while
sharing the validated region and parallel traversal. The separate-input form
retains WRF's four-addition order; the combined form retains its two-addition
order.

The big-step routines differ subtly from `calc_mu_staggered` at a non-periodic
physical endpoint. They evaluate a duplicate-value average instead of directly
copying full mass:

```text
split: 0.5 * (mu + mu + mub + mub)
full:  0.5 * (full_mass + full_mass)
```

Ordinary atmospheric values produce the same mathematical result as a copy,
but the floating-point program is not equivalent. A finite value near
`f32::MAX` overflows during the duplicate addition. Rust preserves this
behavior because output parity includes exceptional finite inputs.

At a periodic physical endpoint, the second operand instead comes from the
adjacent stored halo. Lower periodic boundaries require a preceding halo;
upper periodic boundaries require the already-validated mass point beyond the
domain endpoint. `ColumnMassStaggeringPeriodicity` names the four possible axis
states (`None`, `WestEast`, `SouthNorth`, and `Both`) without a boolean-heavy
public interface. A missing lower halo returns a typed error before either
output is mutated.

## Domain, tile, and memory

`ColumnMassStaggeringRegion` deliberately keeps three coordinate concepts
separate:

1. the field shape describes allocated memory, including any halos;
2. each mass-domain range identifies the physical scalar domain; and
3. each momentum-tile range identifies the points active in this invocation.

All Rust ranges are zero-based and half-open memory offsets. A mass-domain
range mirrors WRF's `ids..ide` or `jds..jde`: its exclusive endpoint is also
the stored upper physical-boundary momentum point. A tile range mirrors WRF's
inclusive `its..ite` or `jts..jte`, so its Rust exclusive endpoint is one
larger. Equality at the lower endpoints means lower-boundary contact; equality
between the tile end and `domain.end + 1` means upper-boundary contact.

The constructor validates every relationship before any field is mutated. It
also derives WRF's cross-axis clipping:

```text
west-east momentum rows = tile_y.start .. min(tile_y.end, domain_y.end)
south-north momentum columns = tile_x.start .. min(tile_x.end, domain_x.end)
```

Values outside those rectangles remain untouched, including allocated halos
and the unused stagger line at the other axis's upper boundary.

## Execution and ownership

All four fields use contiguous west-east-major storage. The two mass fields are
borrowed immutably and the two outputs mutably. Each output pass schedules
complete, disjoint rows on the persistent CPU pool, making standard host
parallelism deterministic and race-free without a field clone or numerical
scratch allocation.

Boundary decisions happen once per row or once around the west-east interior
loop. Interior loops remain contiguous and branch-free. Lightweight `Range`
clones are used to make ownership clear; they contain only two machine words
and do not allocate.

The two output passes remain separate behind `ColumnMassStaggeringKernels` so a
future GPU backend can implement native device kernels and storage rather than
receiving CPU closures.

## CPU performance

The matched benchmark uses a 1,024 × 1,024 physical mass domain and writes
2,099,200 momentum-mass outputs per call. On the Apple M3 Max baseline machine,
portable release Rust measured 332.80 µs with one worker, 115.32 µs with four,
and 242.03 µs with all 16 host workers. Four workers are best because this
streaming kernel reaches memory and heterogeneous-core limits before it can use
every core efficiently.

The exact extracted WRF routine, compiled with GNU Fortran 14.2.0 using
`-O3 -flto`, measured a 286.850 µs median. Rust is 16.0% slower serially and
2.49× faster with four workers than serial Fortran. This is an isolated routine
comparison, not a whole-model speedup claim.

For the doubly periodic `calc_mu_uv` workload, one-worker Rust measured
359.64 µs and matched GNU Fortran 16.1.0 `-O3 -flto` measured a 347.120 µs
median: a 3.6% difference, which is close enough to retain the readable scalar
implementation. Four-worker Rust measured 181.10 µs, or 1.92× faster than the
serial Fortran routine. Sixteen workers measured 400.40 µs and are not useful
for this memory-bound isolated kernel.

After pool and field warm-up, every 100-call phase uses three small scheduler
allocations totaling 4,560 bytes and no reallocations. There is no field-sized
or per-row numerical scratch. The periodic big-step path has the same measured
allocation profile.

Fortran's averaging loops are vectorized. Rust's retained loops are scalar, so
a safe `pulp` prototype was tested. It preserved every tested scalar and
Fortran bit but regressed one- and four-worker performance. A slice-iterator
autovectorization formulation also regressed serial performance. Both were
removed. The full workload, raw samples, confidence intervals, allocation
evidence, and rejected-prototype measurements are in the
[performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/column-mass-staggering-2026-07-13.md).

## Parity evidence

`scripts/run-column-mass-staggering-oracle.sh` extracts the exact
`calc_mu_staggered` body from the pinned
`dyn_em/module_big_step_utilities_em.F`, compiles it without rewriting the
scientific routine, and runs four domain/tile configurations: interior, lower,
upper, and both physical boundaries.

Each case exercises the named path on both axes. The golden file stores all 240
raw IEEE-754 output and sentinel values. Rust compares both complete output
fields for every case, proving exact arithmetic, boundary copies, cross-axis
clipping, and unchanged inactive storage. Separate tests prove validation
before mutation and bitwise equality between one and four CPU workers when the
tile touches all four physical boundaries.

Sixteen seeded cases then cross all four west-east boundary states with all four
south-north states while varying shapes, non-one memory origins, clipping,
signed zero, large finite cancellation, and active NaN/infinity inputs. All
6,150 complete output and sentinel values match: finite values and infinities by
raw bits, NaN by class. Failures identify the seed, output staggering, and first
divergent linear index.

`scripts/run-periodic-column-mass-oracle.sh` separately extracts the exact
`calc_mu_uv` and `calc_mu_uv_1` bodies. Eight cases per routine cover interior,
lower, upper, and both physical-boundary paths; west-east, south-north, and
doubly periodic paths; and the overflow-sensitive physical endpoint. Rust
matches all 960 complete-field output and sentinel values by raw bits. Added
tests cover missing periodic halos, validation before mutation, untouched
storage, and bitwise equality between one and four workers.

This evidence completes focused routine-level coverage for all three column-
mass staggering entry points and randomized coverage for non-periodic
`calc_mu_staggered`. A randomized big-step corpus and an end-to-end ARW
trajectory remain explicit later gates.
