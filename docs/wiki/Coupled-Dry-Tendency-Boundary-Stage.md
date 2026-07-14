# Coupled dry tendency and boundary stage

## Purpose

The ARW solver does not execute dry tendency assembly and boundary assignment
as unrelated utilities. In `dyn_em/solve_em.F`, each large-timestep tile calls
`rk_addtend_dry` and then, for specified or nested domains, `spec_bdy_dry`.
This stage implements that adjacent source order as one Rust capability.

The composition matters because boundary-file values overwrite selected edges
of the newly assembled Runge–Kutta tendencies. Testing either routine alone
cannot catch reversed order, a field mapped to the wrong boundary set, or a
validation failure that occurs after the first routine has already changed
state.

## Source-order data flow

`rk_addtend_dry` combines the current dynamics tendencies with persistent
forward/physics tendencies. On the first Runge–Kutta substep it also adds saved
boundary tendencies. Potential temperature additionally receives diabatic
heating weighted by the full column mass and vertical coefficients. U, V, W,
geopotential, temperature, and column mass each use their WRF C-grid range.

`spec_bdy_dry` then assigns boundary-file tendency values to U, V,
perturbation geopotential, potential temperature, and perturbation column mass.
Nested domains also assign vertical momentum. Global specified domains leave W
as produced by assembly. Periodic X suppresses west/east assignment while
retaining south/north trapezoids.

In short:

```text
RK dynamics + forward/physics + saved first-step values
                         │
                         ▼
                 rk_addtend_dry
                         │ assembled RK tendencies
                         ▼
                   spec_bdy_dry
                         │ selected edge values replaced
                         ▼
             acoustic-step tendency inputs
```

## Rust ownership model

`DryTendencyAssemblyRungeKuttaTendencies` owns the six mutable RK borrows once.
`DryTendencyBoundaryStageInputs` groups forward fields, saved fields,
thermodynamics, map factors, vertical coefficients, and boundary slabs without
allocating or copying field storage. `DryTendencyBoundaryStageControls` groups
the substep phase, boundary widths, and X periodicity.

`DryTendencyBoundaryStageVertical` is deliberately typed:

- `Global` has no W boundary data and cannot accidentally overwrite W;
- `Nested` requires all four W tendency slabs.

`DryTendencyBoundaryStageRegions::try_new` derives the assembly and six
location-specific boundary regions from one physical domain and tile. The
public kernel trait remains narrow enough for a future device-native backend;
it does not expose CPU closures or host slices.

## Cross-routine atomicity

The CPU implementation reborrows the owned fields for validation and checks
both existing kernel contracts before mutation. Only after assembly shapes,
coefficient lengths, every active boundary output, all boundary slab shapes,
widths, and the optional nested-W contract pass does execution begin.

This is stronger than calling the two public kernels naively. In the naive
form, a malformed late W boundary could be detected only after assembly had
changed ten volume fields and column mass. The coupled stage instead returns a
typed `BoundaryAssignment` error with every mutable output unchanged.

## Parity evidence

The oracle extracts the exact pinned `rk_addtend_dry`, `spec_bdy_dry`, and
`spec_bdytend` bodies and runs them in one Fortran executable. Five cases cover:

- first-substep global and nested behavior;
- a later substep with partial horizontal and vertical tiles;
- periodic X with nested W; and
- signed zero, infinity, subnormal, maximum-finite, and NaN behavior.

All 9,360 emitted RK, forward, mass, inactive, and boundary-selected values
match Rust by raw IEEE bits, except NaNs which compare by class. Rust tests also
prove bitwise one/four-worker determinism and late-boundary failure atomicity.

## Performance boundary

On the matched 256 × 256 × 40 first-substep nested workload, optimized serial
Fortran measures 9.029150 ms median. Rust measures 5.8323 ms with four workers
and 3.9960 ms with the default sixteen workers. The readable composition is
therefore retained without extra SIMD or fusion.

## Next integration gate

The next slice prepends the earlier first-step `relax_bdy_dry` call according
to the exact `solve_em.F` ordering. Multi-tile halo exchange and
specified-state updates then surround the verified local acoustic trajectory.
