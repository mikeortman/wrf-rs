# Complete dry boundary-tendency assignment

## Purpose

WRF's `spec_bdy_dry` coordinates the boundary-file tendency assignment that
follows the large-step dry tendency calculation. It applies the already
documented `spec_bdytend` copy operation to every dry prognostic tendency before
the acoustic integration uses those values.

The pinned wrapper lives in `dyn_em/module_bc_em.F`; the scalar operation lives
in `share/module_bc.F`.

## Source order

For one tile, WRF assigns fields in this order:

1. west–east momentum `RU` using the U stagger;
2. south–north momentum `RV` using the V stagger;
3. perturbation geopotential `PH` on full levels;
4. potential-temperature perturbation `T` on half levels;
5. horizontal perturbation column mass `MU`;
6. vertical momentum `RW` on full levels, only for a nested domain.

Each scalar call replaces the contacted outer specified zone with the matching
boundary-file tendency. Periodic X disables west/east assignment while keeping
south/north rows across the periodic span.

## Rust ownership model

`DryBoundaryTendencyKernels` is a narrow backend capability. The associated
field type remains backend-native, so a future GPU implementation can retain
all six outputs and boundary arrays on the device. The CPU implementation uses
the persistent multithreaded backend by default.

Typed groups replace the Fortran wrapper's long positional argument list:

- `DryBoundaryTendencies` owns the five always-active mutable outputs;
- `DryBoundaryTendencyBoundaryFields` groups their oriented boundary data;
- `DryBoundaryVerticalTendency` distinguishes global `Disabled` from a valid
  nested W output and boundary set; and
- `DryBoundaryTendencyRegion` derives every stagger from one physical domain
  and tile description.

The wrapper accepts only boundary tendencies. WRF also passes 24 boundary-state
arrays through `spec_bdy_dry`, but `spec_bdytend` never reads them.

Every active output, all four sides for every field, widths, and staggered
regions are validated before the first assignment. A malformed nested-W north
array therefore cannot leave U through MU partially changed. Execution then
calls the exact scalar capability in source order. There is no unsafe Rust,
numerical scratch, or field clone.

## Vertical behavior

The scalar source has a nonuniform upper-tile rule. U, V, and T begin at the
tile's lower vertical bound but continue to the physical half-level top. PH
and nested W use the supplied upper tile bound because they select full-level
behavior. MU has exactly one horizontal level. The typed region captures these
rules once rather than repeating selector strings at each call.

## Parity evidence

The direct oracle extracts the exact pinned `spec_bdy_dry` and `spec_bdytend`
bodies. Nine cases compare every one of 27,900 stored output values by raw
IEEE-754 bits:

- global and nested domains;
- periodic X;
- southwest and northeast partial tiles;
- a shortened vertical tile that distinguishes half- and full-level rules;
- inactive and zero-width paths; and
- negative zero, subnormal, maximum finite, infinities, and a NaN payload.

Rust-only tests prove complete one/four-worker determinism, preserve every
output on inactive paths, validate all six output roles and all 24 boundary
roles, and demonstrate failure atomicity for a late nested-W error.

## Performance boundary

On the matched 256 × 256 × 40 nested workload, serial Rust and optimized
Fortran both measure about 0.485 ms. Four-worker Rust is 2.64× faster, while
default sixteen-worker Rust is within 3.3% of Fortran. The readable composed
implementation is therefore retained without SIMD or loop fusion.

## Next integration gate

The scalar tendency assignment, complete dry assignment, scalar relaxation,
and complete dry relaxation wrappers are now independently verified. The next
gate composes them with specified-state updates and halo exchange around the
local acoustic trajectory, proving cross-stage ordering over multiple tiles.
