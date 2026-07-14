# Physical boundary-zone assignment

WRF fills a fixed boundary zone after selected dynamical updates. The Rust
`PhysicalBoundaryKernels` capability ports `set_physical_bc3d` and
`set_physical_bc2d` from pinned WRF v4.7.1. It operates directly on caller-owned
XZY volume storage or XY horizontal storage and never allocates field scratch.

## Typed contract

`PhysicalBoundaryRegion` binds one storage shape to the mass-domain,
half-level, and tile ranges. Each horizontal side must store WRF's four-point
boundary zone. Tiles may reach the single upper stagger point, and an interior
tile is a validated exact no-op.

`PhysicalBoundaryVariable` replaces WRF's character selector:

| Rust variable | WRF class | Extra point |
|---|---|---|
| `MassHalfLevel` | `p` or `t` | none |
| `WestEastFace` | `u` | west-east |
| `SouthNorthFace` | `v` | south-north |
| `FullLevel` | `w` | bottom-top |

`PhysicalBoundaryConditions` carries WRF's periodic, symmetric, open,
specified, nested, and polar flags without collapsing combinations. Periodic
axes suppress the corresponding symmetric/open branches. Specified and nested
domains activate all four open copies exactly as the upstream routine does.

## Source order and corners

The implementation preserves the upstream branch order:

1. west-east periodic, symmetric, or open copies for every contacted row;
2. south-north periodic, symmetric, or open copies; and
3. the four explicit doubly periodic corners in the two-dimensional routine.

South-north copies can read values produced by earlier rows, so that pass stays
serial. Benchmarking also showed that scheduling the thin west-east perimeter
through the worker pool costs more than it saves. The complete routine therefore
runs in one allocation-free source-order traversal regardless of the backend's
worker count. Larger numerical stages remain parallel.

The port retains small upstream differences between the two routines, including
the two-dimensional staggered symmetric range beginning at the edge point and
the U/V sign reversal on the component normal to a symmetric boundary.

## Validation and parity

Shape validation finishes before the first store. Malformed input therefore
leaves the destination unchanged. The direct oracle extracts both exact routine
bodies from the pinned source and compares complete storage for:

- periodic mass fields;
- specified U and W fields;
- nested V fields;
- partial and inactive tiles;
- two-dimensional periodic, specified, and nested fields; and
- NaN, infinities, signed zero, and ordinary finite values.

Finite values, infinities, and signed zero match by raw IEEE-754 bits. NaNs
match by class. One-worker and four-worker calls produce identical complete
storage. See the [performance record](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/physical-boundary-2026-07-14.md)
for the matched perimeter benchmark and the decision not to parallelize this
thin operation.

## Integration boundary

The capability models a single-rank patch, so WRF's on-processor periodic tests
are true. It does not exchange MPI halos. The complete acoustic boundary stage
uses it at the exact `solve_em.F` insertion points. That stage rejects polar
configuration because the separate `pxft` polar filter is not yet ported, even
though the standalone copy routine faithfully represents the `polar` flag's
south-north open-copy behavior.
