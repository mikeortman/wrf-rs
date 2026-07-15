# Registry-backed ARW accepted-stage oracle

This fixture is a dependency-closed projection of already selected ARW stages,
not a complete `solve_em` timestep. It preserves the pinned WRF v4.7.1 order
for the included routines:

1. the seven `rk_step_prep` dependency bodies;
2. `rk_addtend_dry` for the first Runge-Kutta pass;
3. `small_step_prep`, initial `calc_p_rho`, `calc_coef_w`, and three acoustic
   iterations of `advance_uv`, `advance_mu_t`, `advance_w`, `sumflux`, and
   closing `calc_p_rho`;
4. `calc_mu_uv_1` and `small_step_finish`, which reconstruct full perturbation
   state before physics; and
5. `moist_physics_prep_em`, the unmodified Kessler routine, and
   `moist_physics_finish_em`.

The fixture uses x-contiguous Fortran `(i,k,j)` storage with six allocated
points per axis: memory bounds `0:5`, domain bounds `1:5`, and a tile covering
`1:5`. Mass-point physics clips the upper stagger to `4`. The WRF packed moist
array retains slot 1 as padding and binds `qv`, `qc`, and `qr` to slots 2, 3,
and 4, matching the Kessler Registry package.

`Registry.model` is the dependency-closed projection of the ordinary state
and Kessler scalar declarations used by this trajectory. Its rank, time-level,
and staggering contracts are checked by `wrf-model`; the upstream
`Registry/Registry.EM_COMMON` checksum is pinned in `wrf-v4.7.1.sha256`.

All inputs are finite deterministic values. Each checkpoint prints
`stage field flat-index raw-f32-bits`; the flat index advances west-east first,
then vertically, then south-north. Complete allocated storage is printed so a
comparison reports the first changed active or inactive value. The script runs
the Fortran projection twice and rejects nondeterministic output.

The projection deliberately omits unported dynamics tendencies, boundary and
halo operations, scalar advection, and the other work between the selected
stages in the full WRF Runge-Kutta loop. It must not be cited as complete
forecast parity.
