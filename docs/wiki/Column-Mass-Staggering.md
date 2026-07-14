# Column-mass staggering

WRF's ARW dynamical core stores dry-air column mass at scalar, or mass-grid,
points. Horizontal momentum components live on an Arakawa C grid: the
west-east component lies between adjacent mass points in the west-east
direction, while the south-north component lies between adjacent rows.
`calc_mu_staggered` constructs the mass factors needed at those two momentum
locations.

## Algorithm

Let `mu` be perturbation dry-air column mass and `mub` the hydrostatic
base-state contribution. At an interior west-east momentum point,

```text
muu(i,j) = 0.5 * (mu(i,j) + mu(i-1,j) + mub(i,j) + mub(i-1,j))
```

The south-north result uses the same arithmetic with row `j-1`:

```text
muv(i,j) = 0.5 * (mu(i,j) + mu(i,j-1) + mub(i,j) + mub(i,j-1))
```

The Rust implementation retains WRF's single precision and source operation
order. It does not first materialize `mu + mub`, because that would change the
rounding sequence and can change output bits.

## Storage and ownership

All four fields use contiguous west-east-major storage. A
`ColumnMassStaggeringRegion` validates the two output rectangles separately:
west-east momentum needs a preceding column, and south-north momentum needs a
preceding row. This makes the halo dependency explicit instead of relying on
Fortran lower bounds.

The two output fields are borrowed mutably and the mass fields immutably. Each
output pass schedules complete, disjoint rows on the persistent CPU pool. No
field clone or numerical scratch allocation occurs in the kernel. The two
passes remain separate so future GPU backends can provide native kernels and
storage rather than accepting host closures.

## Current scope

The implemented slice corresponds to an interior WRF tile, where every output
uses the two-point average. `calc_mu_staggered` also contains four physical
boundary cases that copy the nearest full mass instead of averaging through a
halo. Those branches are explicitly not yet implemented and remain the next
parity gate; the root `README.md` does not count this routine as complete.

## Parity evidence

`scripts/run-column-mass-staggering-oracle.sh` extracts the exact
`calc_mu_staggered` routine body from the pinned
`module_big_step_utilities_em.F`, compiles it with a deterministic interior
fixture, and compares 60 raw IEEE-754 output values. Sentinels around both
active rectangles prove that halos remain untouched. Rust additionally checks
shape failure before mutation and bitwise equality between one and four CPU
workers.

The extraction is necessary because the routine lives inside a large module
whose unrelated procedures depend on much of WRF. The script does not rewrite
the scientific body; the pinned source text is selected verbatim at build time.
