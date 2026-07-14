# Complete dry large-step boundary stage

## Purpose

WRF's large-timestep tile loop does not treat dry boundary relaxation,
tendency assembly, and boundary assignment as unrelated utilities. In pinned
v4.7.1 `dyn_em/solve_em.F` (commit `f52c197ed39d12e087d02c50f412d90d418f6186`),
each tile calls `relax_bdy_dry` only when the domain is specified or nested
and `rk_step == 1`, then `rk_addtend_dry` unconditionally, then `spec_bdy_dry`
when the domain is specified or nested. This stage implements that complete
three-routine sequence as one Rust capability,
`DryLargeStepBoundaryStageKernels`.

The composition matters because the three routines share mutable state.
Testing each verified kernel alone cannot catch a reversed order, a substep
where relaxation ran but assembly skipped the saved fields, or a validation
failure that occurs after an earlier routine has already changed state.

## Source-order data flow

`relax_bdy_dry` adds relaxation forcing into the five saved boundary
tendencies — WRF's `u_save`, `v_save`, `ph_save`, `t_save`, and, on nested
domains, `w_save` — and into the Runge–Kutta column-mass tendency `mu_tend`.

`rk_addtend_dry` immediately reads those saved fields to assemble the RK
tendencies: it combines the current dynamics tendencies with persistent
forward/physics tendencies, adds the saved boundary tendencies on the first
substep, and accumulates the forward column-mass tendency into the same
`mu_tend`. Potential temperature additionally receives diabatic heating
weighted by the full column mass and vertical coefficients.

`spec_bdy_dry` then assigns boundary-file tendency values over selected edges
of the assembled U, V, perturbation geopotential, potential temperature, and
perturbation column-mass tendencies. Nested domains also assign vertical
momentum; periodic X suppresses west/east assignment.

```text
boundary-file values + tendencies
              │
              ▼
       relax_bdy_dry        (first substep, specified or nested)
              │ relaxed saved tendencies + mu_tend
              ▼
      rk_addtend_dry        (every substep)
              │ assembled RK tendencies + mu_tend
              ▼
       spec_bdy_dry         (specified or nested)
              │ selected edge values replaced
              ▼
   acoustic-step tendency inputs
```

## Typed execution modes

`DryLargeStepBoundaryStageMode` makes the substep and domain kind one typed
choice instead of two Booleans that can disagree:

- `FirstSubstepGlobal` carries the relaxation-only inputs;
- `FirstSubstepNested` adds the nested W state and boundary-file arrays;
- `LaterSubstepGlobal` carries nothing extra;
- `LaterSubstepNested` carries only the four W tendency boundary slabs that
  assignment still needs.

Each variant carries exactly the inputs its stages read, so no mode accepts
unused data, relaxation activity and first-substep assembly accumulation
cannot diverge, and no invalid substep/domain combination is representable.

## Rust ownership model

`DryLargeStepSavedTendencies` owns the five mutable saved-tendency borrows
once. The stage hands mutable reborrows to relaxation and immutable reborrows
to assembly, mirroring the Fortran write-then-read dependency without cloning
any field. The RK column-mass tendency flows mutably through all three
routines: relaxation adds forcing, assembly accumulates the forward tendency,
and assignment overwrites its boundary edges.

Relaxation pairs boundary-file values with the same boundary-file tendencies
that `spec_bdy_dry` later assigns. The stage therefore accepts the tendencies
once, in `DryBoundaryTendencyBoundaryFields`, and the first-substep values
separately, in `DryLargeStepRelaxationBoundaryValues`, zipping the two into
per-field relaxation boundary data internally.

`DryLargeStepBoundaryStageRegions::try_new` derives the relaxation, assembly,
and boundary-assignment regions from one physical domain and one active tile,
so the composed stage cannot run its kernels on disagreeing ranges.
`relaxation_workspace_shape()` exposes the caller-owned mass-weighting
workspace covering the tile plus its stencil halo. Execution reuses the three
verified kernels, each parallelized through the persistent CPU pool.

## Cross-routine atomicity

The CPU implementation runs crate-private validators for all three kernels —
including `validate_cpu_dry_boundary_relaxation`, newly exposed from the dry
relaxation module — before the first mutation. Only after the active
relaxation, assembly, and boundary-assignment contracts all pass does any
kernel execute.

This is stronger than calling the three public kernels naively. In the naive
form, an invalid nested-W assignment boundary would be detected only after
relaxation and assembly had already changed the saved tendencies, ten volume
fields, and column mass. The composed stage instead returns a typed
`BoundaryAssignment` error with every RK output, every saved tendency, and
the mass-weighting workspace untouched.

## Parity evidence

The oracle compiles one direct Fortran executable that extracts, in pinned
source order, `relax_bdytend`, `relax_bdytend_tile`, and `relax_bdytend_core`
plus `spec_bdytend` from `share/module_bc.F`, `relax_bdy_dry` and
`mass_weight` plus `spec_bdy_dry` from `dyn_em/module_bc_em.F`, and
`rk_addtend_dry` from `dyn_em/module_em.F`. Its cases cover:

- first-substep global specified and first-substep nested behavior;
- a later nested substep, proving relaxation is skipped;
- periodic X;
- opposite partial tiles, an inactive tile, and an empty relaxation band; and
- signed zero, infinities, subnormal, maximum-finite, and NaN behavior.

Each case compares the complete RK, forward, saved, and column-mass storage,
so an edge written by the wrong routine or in the wrong order cannot pass.

## Performance boundary

The matched 256 × 256 × 40 first-substep nested workload is defined for both
implementations: the Fortran harness compiles the same pinned routines around
a benchmark driver, and the Rust side runs the composed stage on identical
inputs. Measured results are owned by the benchmark tracking records and CI
artifacts rather than this page.

## Next integration gate

This stage completes the dry large-timestep boundary sequence around
`rk_addtend_dry`. The next slices surround the verified local acoustic
trajectory with multi-tile halo exchange and specified-state updates,
building toward the full `solve_em.F` tile loop.
