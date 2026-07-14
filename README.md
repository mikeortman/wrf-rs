# wrf-rs

An incremental, test-driven Rust reimplementation of the Weather Research and
Forecasting (WRF) model.

## Port provenance

At this stage, this repository's WRF port—including the Rust implementation,
Fortran parity harnesses, tests, benchmarks, findings, and technical
documentation—is being produced by **OpenAI 5.6 Sol in High mode** under
explicit project instructions to prioritize:

1. correct WRF-compatible output and explainable numerical parity;
2. release-mode computational performance and conservative memory use;
3. safe, idiomatic Rust with no local unsafe code;
4. readable ownership boundaries, descriptive names, and logical modules;
5. upstream-derived and adversarial tests, matched Fortran benchmarks, and
   evidence-gated SIMD; and
6. durable rustdoc, wiki documentation, project state, and upstream findings.

AI authorship is provenance, not proof of correctness. A port slice is accepted
only through its pinned Fortran or upstream-test evidence, added gap coverage,
release verification, and issue-linked pull request history.

This is not yet a weather model. WRF v4.7.1 contains roughly 1,750 Fortran
translation units, generated registry code, external libraries, distributed
memory infrastructure, physics suites, dynamics, data assimilation, chemistry,
and coupled components. The port is therefore organized as independently
verifiable compatibility slices. A slice is complete only when its translated
upstream tests and differential fixtures pass.

## Current state

- The official WRF v4.7.1 source is pinned and fetched into `upstream/WRF`.
- `wrf-time` implements the first compatibility slice: proleptic Gregorian
  time, rational-second intervals, arithmetic, formatting, and model clocks.
- `wrf-compute` provides contiguous scalar fields and a persistent,
  work-stealing CPU pool that uses host parallelism by default.
- `wrf-registry` parses the first typed WRF Registry subset and reproduces
  selected generated state, namelist, dimension-order, and metadata artifacts
  exactly from an ARW-shaped fixture.
- `wrf-domain` provides typed domain, patch, memory, and tile bounds; exact
  RSL_LITE decomposition; and deterministic Y-then-X halo plans.
- `wrf-domain-mpi` applies the same plans with non-blocking MPI while storing
  only the calling rank's patch.
- `wrf-dynamics` contains line-parallel, scratch-free ports of WRF's
  positive-definite correction, Held-Suarez momentum damping, and column-mass
  staggering, including periodic `calc_mu_uv` and combined-mass `calc_mu_uv_1`
  paths, three-component mass/map-factor momentum coupling, and complete-column
  dry-air omega diagnosis, moisture coefficients on all three momentum
  staggers, full inverse density, and full geopotential at pressure points.
  A typed `rk_step_prep` pipeline now composes all seven diagnostics after one
  failure-atomic preflight, including the previously missing communicated-tile
  `calculate_full` stage. Safe dry large-timestep tendency assembly now ports
  `rk_addtend_dry`, including first-substep saved tendencies, later-substep
  reuse, diabatic heating, and every component-specific C-grid bound. Typed
  acoustic small-step preparation now ports `small_step_prep`, including
  first/later time-level handling, coupled perturbation variables, horizontal
  staggers, and the complete full-level column. Acoustic pressure diagnosis now
  ports `calc_p_rho` for nonhydrostatic and hydrostatic modes, initialization
  and forward damping, and the hydrostatic geopotential recurrence.
  Vertical acoustic coefficient construction now ports `calc_coef_w`, including
  the complete-column tridiagonal factorization and rigid/nonrigid top
  boundaries. Horizontal acoustic momentum now ports `advance_uv`, including
  split pressure gradients, divergence damping, governing modes, relaxation,
  periodic, physical-edge, and polar paths. Acoustic column mass, vertical mass
  flux, and perturbation potential temperature now port `advance_mu_t`, with
  complete-column continuity integration and horizontal/vertical theta transport.
  Implicit acoustic vertical momentum and geopotential now port `advance_w`,
  including both vertical-advection discretizations, terrain reconstruction,
  rigid/nonrigid tops, the tridiagonal solve, and upper damping.
  Acoustic time-averaged mass fluxes now port `sumflux`, including first-step
  tile clearing, all three stagger-specific ranges, and final linear recoupling.
  `AcousticTrajectoryKernels` composes the complete local nonhydrostatic chain
  with one preflight boundary; a direct three-substep WRF oracle matches all
  2,196 selected final state and diagnostic values bit-for-bit. Specified-zone
  tendency updates now port `spec_bdyupdate` for mass, U, V, horizontal-mass,
  and full-level fields, including periodic X and WRF's trapezoidal corners.
  Mass-normalized geopotential boundary updates now port `spec_bdyupdate_ph`
  with the same geometry and exact source-order arithmetic. Zero-gradient
  specified boundaries now port `zero_grad_bdy`, including nearest-interior
  copies, periodic X, partial tiles, and WRF's complete-to-domain-top vertical
  traversal. Flow-dependent scalar boundaries now port `flow_dep_bdy`, using
  coupled U/V signs to copy nearest-interior outflow and clear inflow for
  moisture, TKE, tracer, and scalar callers. Typed constant and preserve
  policies also port `flow_dep_bdy_qnn` and `flow_dep_bdy_fixed_inflow` through
  the same verified traversal.
  Deterministic fixtures and seeded randomized corpora check upstream `REAL`
  bit patterns.
- `wrf-physics` contains the first physical parameterization: parallel Kessler
  warm-rain microphysics with reusable scratch and exact pinned-Fortran output
  parity.
- `wrf-io` defines the first typed ARW NetCDF/restart schema, writes borrowed
  fields in classic 64-bit-offset format, reads NetCDF-3/4 into caller storage,
  and compares restart metadata and field bits with bounded scratch.
- CPU SIMD is selected per translated kernel after scalar parity; see
  `docs/architecture/simd.md`.
- Scientific source families own nested modules instead of flattening every
  implementation file at the crate root; see
  `docs/architecture/module_structure.md`.
- Rust tests cite the corresponding cases in
  `upstream/WRF/external/esmf_time_f90/Test1.F90`.
- The port-status table below is the source of truth for implementation scope.
- The [GitHub Wiki](https://github.com/mikeortman/wrf-rs/wiki) publishes the
  technical encyclopedia maintained under [`docs/wiki`](docs/wiki/README.md).
- [`UPSTREAM_FINDINGS.md`](UPSTREAM_FINDINGS.md) records reproducible WRF bugs,
  test gaps, and performance opportunities suitable for upstream reporting.
- [`TEST_COVERAGE.md`](TEST_COVERAGE.md) tracks what is tested and what still
  needs adversarial coverage.
- [`PERFORMANCE_PARITY.md`](PERFORMANCE_PARITY.md) tracks matched Rust and
  optimized-Fortran kernel performance without extrapolating to whole-model
  speedup.

## Port status

Target: WRF v4.7.1 at commit
`f52c197ed39d12e087d02c50f412d90d418f6186`.

The states below describe translated interfaces and tests, not scientific
accuracy. Whole-model parity remains 0% until a Rust executable can consume a
WRF initialization and match an upstream integration.

| Area | State | Evidence | Next gate |
|---|---|---|---|
| Source pin and license | Complete | `UPSTREAM.toml`, `scripts/fetch-wrf.sh` | Monitor upstream only by explicit retargeting |
| ESMF-derived time/calendar | Complete for active Test1 surface | 93/93 active cases; both Fortran interfaces match the golden output | Add cases when later WRF callers expose untested behavior |
| Registry/configuration | In progress | Typed `dimspec`, `state`, and `rconfig` parser with physical source locations; six WRF-generated artifact goldens match exactly | Add includes/conditionals, packages, typedefs, communication entries, and the remaining generators |
| Domain decomposition / halo exchange | In progress | Exact `task_for_point.c` decomposition; direct `period.c` periodic/stagger parity; deterministic local and four-rank MPI results match | Add generated communication descriptors, multi-field aggregation, nesting, and broader process grids |
| ARW dynamical core | In progress | Positive-definite sheet/slab, Held-Suarez damping, all three column-mass staggering entry points, integrated failure-atomic `rk_step_prep`, `rk_addtend_dry`, the complete seven-kernel local acoustic trajectory, `spec_bdyupdate`, `spec_bdyupdate_ph`, `zero_grad_bdy`, and all three `flow_dep_bdy` policies have direct Fortran evidence; the trajectory matches 2,196 selected final values and 13,400 boundary fixture values match exactly or by NaN class | Port remaining relaxation stages, insert boundary/halo work around the local acoustic trajectory, then couple it to the large-step tendency path |
| Physics drivers and schemes | In progress | Kessler warm-rain microphysics matches all 660 mutable oracle values exactly; one/four-worker determinism, reusable scratch, matched optimized-Fortran benchmark, and allocation evidence | Port microphysics driver/state mapping and add a coupled precipitation trajectory |
| I/O and NetCDF metadata | In progress | Typed minimum ARW schema; independent NetCDF-C/Rust restart files match ordered metadata and every field bit | Add full Registry-selected state, alarm metadata, NetCDF-4 output policy, and resumed-trajectory parity |
| WRFDA, WRF-Chem, WRF-Hydro, TL/adjoint | Not started | — | Separate workstreams after ARW baseline |
| End-to-end idealized/regression suite | Not started | — | `em_b_wave`/`em_squall2d_x` differential runs |

Parity means exact discrete behavior; per-routine numerical fixtures with an
explicit exact or ULP policy; per-timestep trajectory comparisons including
restart equivalence; and ultimately serial/distributed operational parity.
Passing a looser end-state tolerance does not excuse an unexplained divergent
trajectory.

Work remaining is tracked in [GitHub Issues](https://github.com/mikeortman/wrf-rs/issues).

Each implementation starts from one issue on an `issue-N` branch, lands through
one pull request whose description closes that issue, and uses auto-merge only
after required Rust and Fortran parity checks pass. This preserves the issue,
review diff, CI evidence, commit, and merge as one traceable chain.

## Reproduce

```sh
./scripts/fetch-wrf.sh
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
./scripts/run-wrf-time-oracle.sh
./scripts/run-positive-definite-oracle.sh
./scripts/run-held-suarez-oracle.sh
./scripts/run-column-mass-staggering-oracle.sh
./scripts/run-periodic-column-mass-oracle.sh
./scripts/run-momentum-coupling-oracle.sh
./scripts/run-omega-diagnosis-oracle.sh
./scripts/run-moisture-coefficient-oracle.sh
./scripts/run-inverse-density-oracle.sh
./scripts/run-pressure-point-geopotential-oracle.sh
./scripts/run-runge-kutta-preparation-oracle.sh
./scripts/run-dry-tendency-assembly-oracle.sh
./scripts/run-acoustic-step-preparation-oracle.sh
./scripts/run-acoustic-pressure-oracle.sh
./scripts/run-specified-boundary-update-oracle.sh
./scripts/run-specified-boundary-geopotential-oracle.sh
./scripts/run-zero-gradient-boundary-oracle.sh
./scripts/run-flow-dependent-boundary-oracle.sh
./scripts/run-flow-dependent-inflow-policies-oracle.sh
./scripts/run-vertical-acoustic-coefficients-oracle.sh
./scripts/run-acoustic-horizontal-momentum-oracle.sh
./scripts/run-acoustic-mass-theta-oracle.sh
./scripts/run-acoustic-vertical-momentum-oracle.sh
./scripts/run-acoustic-flux-accumulation-oracle.sh
./scripts/run-acoustic-trajectory-oracle.sh
./scripts/randomized-arw/run-oracles.sh
./scripts/run-registry-oracle.sh
./scripts/run-domain-topology-oracle.sh
./scripts/run-clipped-tiles-oracle.sh
./scripts/run-mpi-halo-parity.sh
./scripts/run-periodic-halo-oracle.sh
./scripts/run-kessler-oracle.sh
./scripts/run-netcdf-restart-oracle.sh
./scripts/benchmark-held-suarez-fortran.sh
./scripts/benchmark-positive-definite-fortran.sh
./scripts/benchmark-column-mass-staggering-fortran.sh
./scripts/benchmark-periodic-column-mass-fortran.sh
./scripts/benchmark-momentum-coupling-fortran.sh
./scripts/benchmark-omega-diagnosis-fortran.sh
./scripts/benchmark-moisture-coefficients-fortran.sh
./scripts/benchmark-inverse-density-fortran.sh
./scripts/benchmark-pressure-point-geopotential-fortran.sh
./scripts/benchmark-runge-kutta-preparation-fortran.sh
./scripts/benchmark-dry-tendency-assembly-fortran.sh
./scripts/benchmark-acoustic-step-preparation-fortran.sh
./scripts/benchmark-acoustic-pressure-fortran.sh
./scripts/benchmark-vertical-acoustic-coefficients-fortran.sh
./scripts/benchmark-acoustic-horizontal-momentum-fortran.sh
./scripts/benchmark-acoustic-mass-theta-fortran.sh
./scripts/benchmark-acoustic-vertical-momentum-fortran.sh
./scripts/benchmark-acoustic-flux-accumulation-fortran.sh
./scripts/benchmark-zero-gradient-boundary-fortran.sh
./scripts/benchmark-flow-dependent-boundary-fortran.sh
./scripts/benchmark-flow-dependent-inflow-policies-fortran.sh
./scripts/benchmark-kessler-fortran.sh
./scripts/benchmark-netcdf-restart.sh 1000
cargo bench -p wrf-dynamics --bench column_mass_staggering -- --noplot
cargo bench -p wrf-dynamics --bench momentum_coupling -- --noplot
cargo bench -p wrf-dynamics --bench omega_diagnosis -- --noplot
cargo bench -p wrf-dynamics --bench moisture_coefficients -- --noplot
cargo bench -p wrf-dynamics --bench inverse_density -- --noplot
cargo bench -p wrf-dynamics --bench pressure_point_geopotential -- --noplot
cargo bench -p wrf-dynamics --bench runge_kutta_preparation -- --noplot
cargo bench -p wrf-dynamics --bench dry_tendency_assembly -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_step_preparation -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_pressure -- --noplot
cargo bench -p wrf-dynamics --bench vertical_acoustic_coefficients -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_horizontal_momentum -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_mass_theta -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_vertical_momentum -- --noplot
cargo bench -p wrf-dynamics --bench acoustic_flux_accumulation -- --noplot
cargo bench -p wrf-dynamics --bench zero_gradient_boundary -- --noplot
cargo bench -p wrf-dynamics --bench flow_dependent_boundary -- --noplot
cargo bench -p wrf-dynamics --bench flow_dependent_inflow_policies -- --noplot
cargo bench -p wrf-physics --bench kessler_microphysics -- --noplot
```

The upstream source is intentionally ignored by the root repository. Its tag,
commit, archive URL, and SHA-256 digest are checked by the fetch script.

## Parity policy

1. Pin an upstream routine and its tests before translating it.
2. Preserve numerical ordering unless a documented Rust design requires a
   change; floating-point reassociation can change model output.
3. Port upstream tests as named cases and keep their provenance visible.
4. Add differential tests that run Fortran and Rust from identical fixtures.
5. Require exact results for discrete code and explicitly justified tolerances
   for floating-point and NetCDF fields.
6. Never mark a subsystem complete from compilation alone.

The port targets semantic and output parity, not line-by-line transliteration.
Rust implementations should use safe ownership, typed invariants, in-place
algorithms, and trusted ecosystem crates where they improve clarity or
performance without changing the required output.

WRF's upstream public-domain notice is retained in
`upstream/WRF/LICENSE.txt` after fetching.
