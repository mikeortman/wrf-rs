# Complete dry boundary relaxation

## Purpose

WRF's `relax_bdy_dry` is the dry-dynamics boundary-relaxation coordinator. It
does not introduce a new stencil. Instead, it prepares each prognostic field
in the units expected by `relax_bdytend`, calls that verified scalar kernel,
and preserves an order that matters to the surrounding Runge–Kutta stage.

The pinned implementation lives in `dyn_em/module_bc_em.F`. Its dependencies
are `mass_weight` in the same file and `relax_bdytend` in `share/module_bc.F`.

## Execution order

For one tile, WRF performs these operations:

1. relax west–east momentum `U`;
2. relax south–north momentum `V`;
3. mass-weight geopotential `PH` on full vertical levels, then relax it;
4. mass-weight potential-temperature perturbation `T` on half levels, then
   relax it;
5. relax perturbation column mass `MU` as a horizontal field;
6. for a nested domain only, mass-weight vertical velocity `W` on full levels
   and relax it.

Global domains omit the final W operation. The Rust API makes that choice a
typed `Global` or `Nested` variant instead of a Boolean plus conditionally
valid pointers.

## Mass weighting

The scalar relaxation kernel expects mass-coupled PH, T, and W values. WRF
constructs a scratch value from the local full column mass and two adjacent
vertical levels:

```text
full level: scratch(k) = full_mass * (cf1(k) * field(k) + cf2(k) * field(k-1))
half level: scratch(k) = full_mass * (cf1h(k) * field(k) + cf2h(k) * field(k+1))
```

The exact coefficient names and boundary treatment follow the pinned Fortran.
The arithmetic order is retained so finite results match by raw IEEE-754 bits.
Full-level inputs need the lower neighboring level; half-level inputs need the
upper neighboring level.

## Rust structure

The implementation is grouped under
`specified_boundary_update::dry_relaxation`, with smaller modules for fields,
boundaries, coefficients, region geometry, vertical policy, tendencies,
workspace, errors, and CPU execution. `DryBoundaryRelaxationKernels` is the
narrow backend capability; backend-native storage stays behind its associated
field type so a future GPU implementation need not stage whole fields through
the host.

The caller owns one workspace covering the tile plus the horizontal neighbor
required by the five-point stencil. PH, T, and optional W reuse that allocation
in sequence. This avoids WRF's large automatic scratch allocation per tile and
thread while keeping ownership explicit and safe. No unsafe Rust is used.

Before the first tendency changes, the CPU implementation validates all active
state, tendency, boundary, coefficient, workspace, region, and stencil
contracts. A late W-boundary error therefore cannot leave U through MU partly
updated. Work is parallelized by disjoint south–north planes through the
persistent CPU pool.

Inactive tiles and empty relaxation bands return without mass weighting. The
Fortran scratch contents are local and never observed on these paths, so this
is an exact output-preserving optimization.

## Parity evidence

The direct oracle extracts the pinned `relax_bdy_dry`, `mass_weight`, and scalar
relaxation code rather than rewriting an expected-value formula. Eight cases
compare every one of 24,800 stored tendency values:

- global and nested domains;
- periodic X;
- southwest and northeast partial tiles;
- inactive and empty relaxation bands; and
- signed zero, subnormal, maximum finite, infinity, and NaN behavior.

Finite values and infinities compare by raw bits. NaNs compare by class because
their payload and sign propagation vary across otherwise conforming compiler
and platform combinations. Rust-only tests also prove one/four-worker complete
output determinism, validation of all 13 field roles before mutation, and
workspace preservation on inactive paths.

## Performance boundary

On the matched 256 × 256 × 40 nested workload, default sixteen-worker Rust
measures 4.1620 ms versus optimized serial Fortran's 4.1218 ms median, a 1%
difference. The Rust call reuses caller workspace and makes no field-sized
allocation. That is close enough under the project rule, so SIMD and more
complex specialization are deferred until an integrated profile proves a need.

## Next integration gate

The remaining dry-boundary wrapper is `spec_bdy_dry`. After it is ported, the
complete boundary-file assignment, dry relaxation, boundary update, and halo
sequence can be inserted around the verified local acoustic trajectory. That
coupled fixture is the evidence needed for ordering across tiles and substeps.
