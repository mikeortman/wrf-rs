# Current implementation state

Last updated: 2026-07-14

This file is the durable handoff for continuing the WRF Rust port after context
compaction or a new session. Update it only with verified current-state facts.

## Goal and non-negotiable constraints

- Reimplement WRF in Rust and prove parity by porting and differentially running
  upstream tests.
- Follow `RUST_BACKEND_STYLE_GUIDE.md` as the governing Rust contract.
- Implement CPU kernels first.
- Multithreading is the standard runtime path, not an opt-in feature.
- Keep storage and kernel capability boundaries compatible with a future native
  GPU backend.
- Optimize release-mode compute and memory behavior without sacrificing clear
  ownership, names, and structure. Cheap descriptor/`Arc` clones are acceptable;
  field and domain-state clones in hot paths are not.
- Treat SIMD as a per-kernel CPU implementation layer after scalar parity.
- Group source by scientific family with nested modules; keep crate roots as
  stable public facades rather than flat implementation indexes.
- Benchmark the exact pinned Fortran routine on a matched workload for every
  numerical kernel and track honest Rust/Fortran ratios.
- Stop performance tuning when matched Rust and Fortran implementations are
  operationally close unless a measured model-level bottleneck justifies more
  complexity.

## Upstream reference

- Repository: `https://github.com/wrf-model/WRF`
- Tag: `v4.7.1`
- Commit: `f52c197ed39d12e087d02c50f412d90d418f6186`
- Archive SHA-256:
  `7227916c7871cec36a0a1bf23619fe6d29664474679c8207b4c6f22b10cbab6b`
- Local source: `upstream/WRF` (ignored by the root Git repository)
- Reproducer: `scripts/fetch-wrf.sh`

The GitHub release archive omits WRF's nested submodule contents. Their exact
commits and archive checksums are recorded in `UPSTREAM.toml`, and
`scripts/fetch-wrf.sh` fetches them into `phys/noahmp`, `phys/MYNN-EDMF`, and
`.ci/hpc-workflows`.

## Implemented crates

### `wrf-time`

Implemented:

- proleptic Gregorian dates including year zero and negative years;
- exact rational-second model timestamps and fixed intervals;
- normalized WRF-compatible formatting;
- time/interval addition, subtraction, scaling, and truncating ratios;
- deterministic fixed-step clocks;
- typed errors and component types.

Tests are at the bottom of their implementation files per the style guide. A
previous false expectation for negative fractional interval formatting was
corrected to WRF's `-NN/DD` convention.

### `wrf-compute`

Implemented:

- checked three-dimensional grid shapes;
- contiguous copyable-scalar CPU fields;
- backend-owned field storage boundary;
- narrow `ComputeBackend` allocation trait;
- persistent Rayon CPU thread pool using all available host parallelism by
  default;
- disjoint mutable output chunks with immutable shared inputs;
- typed kernel and worker failures;
- cheap backend clones sharing one `Arc<ThreadPool>`.
- exact contiguous-block scheduling for indivisible grid lines and profiles;
- safe paired-output block scheduling for one-pass updates to two owned fields;
- typed validation of block shapes and worker-panic containment.

Future numerical crates must define narrow kernel capability traits. Do not put
unrelated kernels into `ComputeBackend` and do not expose arbitrary CPU closures
as the GPU API.

### `wrf-registry`

Implemented:

- a safe typed parser for dependency-closed `dimspec`, `state`, and `rconfig`
  entries;
- WRF-compatible backslash continuation, case folding, quoted-token, comment,
  and quoted-`#` behavior;
- physical source locations and typed diagnostics for malformed input;
- typed standard, namelist, and constant dimension bounds, coordinate axes,
  value types, state dimensions/modifiers, staggering flags, time levels, and
  scalar/expression-sized namelist entries;
- source-order resolution of state dimension references;
- a separate selected-artifact generator, with no runtime domain ownership in
  the parser crate;
- byte-identical `state_struct.inc`, `namelist_defines.inc`,
  `namelist_defaults.inc`, `namelist_statements.inc`, and
  `model_data_order.inc` output against WRF v4.7.1;
- exact normalized state metadata for regular time levels and generated
  boundary/boundary-tendency arrays, derived from WRF `allocs_*.F` artifacts;
- committed upstream goldens and a reproducible `scripts/run-registry-oracle.sh`
  differential gate.

The first fixture is deliberately small but uses real ARW `t` and `mu` entry
forms, including continuations, complex I/O specifications, two time levels,
boundary modifiers, every dimension-length mode, and scalar/vector runtime
configuration storage. Includes, conditionals, `typedef`, `i1`, packages,
communication entries, four-dimensional scalar-array generation, and the
remaining generated files are explicitly not yet supported. See
`docs/wiki/WRF-Registry.md`.

### `wrf-domain` and `wrf-domain-mpi`

Implemented:

- signed zero-based half-open domain indices with checked Fortran conversion;
- typed physical domain, horizontal, patch, memory, and tile bounds;
- exact centered-remainder RSL_LITE decomposition and row-major patch IDs;
- WRF guard-point memory bounds with explicit physical-boundary storage;
- edge-tile halo extension followed by physical-domain clipping;
- transport-neutral Y-then-X halo plans with internal corners, periodic
  endpoints, and per-axis field staggering;
- deterministic local execution using boundary-sized staged messages;
- one-patch XZY storage for rank-local transport without field-sized clones;
- a separate safe MPI adapter using receives-before-sends non-blocking phases;
- direct pinned `task_for_point.c` and `period.c`/`f_pack.F90` differential
  oracles; and
- complete four-rank MPI versus local-memory parity for nonperiodic and doubly
  periodic staggered cases.

Topology is setup work and no misleading throughput ratio is recorded. A
matched halo benchmark waits for WRF-compatible multi-field aggregation.

### `wrf-physics`

Implemented:

- the first dependency-closed physical parameterization, WRF Kessler warm-rain
  microphysics from pinned `phys/module_mp_kessler.F`;
- a focused `KesslerMicrophysicsKernels` capability with backend-owned field
  and workspace associated types for future GPU implementations;
- validated scalar parameters, field shapes, precipitation shapes, horizontal
  tile ranges, the surface-start vertical contract, and minimum level count;
- sedimentation, adaptive fallout substeps, surface precipitation,
  cloud-to-rain conversion, saturation adjustment, evaporation, and latent
  heating with WRF single-precision operation order;
- default parallel execution across independent south-north rows on the
  persistent CPU pool;
- reusable production scratch plus one vertical terminal-velocity buffer per
  worker, with no field clones and no numerical scratch allocation per call;
- exact raw-bit equality for all 660 mutable output, halo, and precipitation
  values in a direct pinned-Fortran oracle; and
- typed failure-before-mutation tests, one/four-worker determinism, unchanged
  halo tests, matched optimized-Fortran timing, and allocation evidence.

The candidate inventory, dependency map, selection rationale, exclusions, and
parity policy are in `docs/physics/kessler-selection.md`. The next physics gate
is microphysics driver and Registry moisture-species integration, followed by a
coupled precipitation trajectory.

### `wrf-dynamics`

Implemented:

- the focused `PositiveDefiniteKernels` backend capability;
- WRF `positive_definite_sheet` for single-precision CPU fields;
- WRF `positive_definite_slab` with typed, validated active ranges;
- in-place correction with no temporary line allocation or copies;
- independent west-east lines scheduled through the persistent Rayon pool;
- scalar ordered minimum/sum reductions for exact parity;
- typed sheet-shape, total-count, and worker failures.
- the focused `HeldSuarezDampingKernels` backend capability;
- WRF `held_suarez_damp` over six borrowed fields with no field clones;
- typed validation of shapes, ranges, the pressure reference level, and both
  C-grid staggered predecessors;
- exact-bit tendency updates over independently scheduled west-east lines;
- scientific-family directories for `positive_definite` and `held_suarez`,
  with `lib.rs` retained as the stable public facade.
- all interior, lower-boundary, upper-boundary, and both-boundaries paths of
  ARW `calc_mu_staggered`, exposed through `ColumnMassStaggeringKernels`;
- typed separation of allocated memory shape, physical mass-domain ranges, and
  active momentum-tile ranges, with boundary contact and cross-axis clipping
  derived rather than supplied as boolean flags;
- parallel, disjoint west-east-major output rows with immutable shared mass
  inputs and no field-sized scratch;
- exact-body Fortran extraction oracle with 240 raw-bit output/sentinel checks
  across all eight axis/path combinations, plus validation-before-mutation and
  one/four-worker determinism tests at all physical boundaries;
- exact `calc_mu_uv` and `calc_mu_uv_1` big-step variants with a typed four-state
  periodicity policy, preserved endpoint expression order, and no numerical
  scratch;
- a 960-value raw-bit oracle covering both input forms, every physical and
  periodic boundary path, inactive storage, and an overflow-sensitive finite
  endpoint;
- WRF `couple_momentum` behind a focused `MomentumCouplingKernels` capability,
  with role-specific borrowed output, velocity, mass, map-factor, and
  coefficient bundles;
- typed C-grid domain/tile validation that derives the distinct U, V, and W
  half/full-level clipping without caller booleans;
- three safe row-parallel, scratch-free output passes whose equal-length slices
  preserve source expression order and enable bounds-check elimination;
- a 3,780-value exact Fortran oracle covering each upper stagger, negative and
  non-one memory origins, untouched storage, finite overflow, and zero map
  factors;
- WRF `calc_ww_cp` behind a focused `OmegaDiagnosisKernels` capability with
  typed velocity, mass, map-factor, coefficient, grid-metric, and complete-column
  inputs;
- safe south-north plane parallelism, validated west-east row views, and an
  output-as-scratch vertical reduction that removes WRF's four automatic tile
  arrays;
- a 1,960-value direct Fortran oracle covering interior and upper horizontal
  clipping, negative/non-one memory origins, untouched storage, finite
  overflow, and zero map factors;
- WRF `calc_cq` behind a focused `MoistureCoefficientKernels` capability with
  typed, non-aliasing outputs, an ordered active-species view, and validated
  component-specific stagger ranges;
- safe row-parallel west-east, south-north, and vertical passes that use each
  output row as species-total scratch and omit WRF's generated padding slot;
- an 8,232-value direct Fortran oracle covering zero, one, and three active
  species, upper staggers, non-one/negative origins, untouched storage, signed
  zero, and finite overflow;
- WRF `calc_alt` behind a focused `InverseDensityKernels` capability with typed
  mass-domain/tile validation and explicit perturbation, base-state, and full
  inverse-density roles;
- a safe row-parallel, scratch-free CPU implementation whose contiguous slices
  preserve WRF's addition order and future device-storage boundary;
- a 2,352-value direct Fortran oracle covering independent and combined upper
  clipping, non-one/negative origins, untouched storage, overflow, cancellation,
  signed zero, and opposite infinities;
- WRF `calc_php` behind a focused `PressurePointGeopotentialKernels` capability
  with typed pressure-point output, full-level inputs, and an explicit upper
  vertical-neighbor contract;
- safe row-parallel averaging over four equal-length input slices that preserves
  WRF's base-state-first operation order without numerical scratch;
- a 2,352-value direct Fortran oracle covering independent and combined upper
  clipping, non-one/negative origins, untouched storage, source-order overflow,
  signed zero, and opposite infinities;
- a typed `RungeKuttaPreparationKernels` capability that performs one all-stage
  preflight and then composes `calculate_full`, `calc_mu_uv`, momentum coupling,
  omega, moisture coefficients, inverse density, and geopotential in WRF order;
- grouped borrowed inputs and non-aliasing outputs, backend-native storage, no
  full-field clones, and an explicit lower-halo contract for the previously
  missing communicated-tile full column mass;
- a coupled 1,728-value exact Fortran oracle over all twelve intermediate and
  final fields, final-stage failure atomicity, one/four-worker determinism,
  matched optimized timing, and warmed allocation evidence;
- a versioned SplitMix64 corpus generator, shared raw-bit input files, and
  pinned Fortran drivers for 68 seeded cases and 39,588 complete outputs across
  all four translated routines;
- exact finite/infinity output comparison, explicit NaN-class policy, and
  first-divergence reports containing seed, field, and linear index.
- WRF `small_step_prep` behind a focused
  `AcousticStepPreparationKernels` capability with typed previous/current time
  levels, saved state, mass roles, diagnostics, map factors, coefficients,
  phase, and region;
- complete preflight of all 24 mutable fields, coefficient lengths, horizontal
  staggers, the full acoustic column, and later-substep lower neighbors before
  any mutation;
- safe row-parallel horizontal and volume modules using paired saved/current
  passes, no numerical scratch, no field clones, and backend-native storage;
- a 9,936-value exact extracted-Fortran oracle covering first and later
  substeps, U/V/W/geopotential staggers, interior lower neighbors, inactive
  sentinels, and exceptional IEEE arithmetic;
- WRF `calc_p_rho` behind a focused `AcousticPressureKernels` capability with
  typed mutable state, thermodynamics, masses, half-level coefficients,
  vertical metrics, governing mode, damping phase, and clipped region;
- distinct nonhydrostatic and hydrostatic modules, including an upward
  geopotential recurrence parallelized by owned south-north planes with
  level-major contiguous-X traversal;
- a 3,456-value exact extracted-Fortran oracle covering both governing modes,
  initialization and forward damping, full and partial vertical tiles,
  inactive sentinels, and exceptional IEEE arithmetic;
- WRF `calc_coef_w` behind a focused
  `VerticalAcousticCoefficientKernels` capability with typed tridiagonal
  outputs, mass/moisture/pressure inputs, hybrid-coordinate coefficients,
  vertical metrics, scalar controls, top-boundary policy, and complete-column
  region;
- safe two-phase south-north plane parallelism with level-major contiguous-X
  traversal, paired forward-elimination outputs, no numerical scratch, and no
  field clones;
- a 3,024-value exact extracted-Fortran oracle covering rigid/nonrigid top
  boundaries, full and partial horizontal tiles, negative/non-one memory
  origins, inactive sentinels, and exceptional IEEE arithmetic;

The differential drivers compile the pinned upstream Fortran module directly.
The sheet covers nine exact-bit cases, including signed zero and the epsilon
branch. The slab fixture covers non-one memory origins, domain/tile clipping,
correction branches, and unchanged halo/stagger sentinels. Rust also proves
sheet bitwise determinism between one and four workers.

The Held-Suarez differential fixture checks 16 active and boundary values with
non-one memory origins. Added Rust tests cover one/four-worker determinism,
shape failure before mutation, all range categories, staggered predecessors,
and the pressure reference level.

The three column-mass staggering entry points now have focused routine-level
coverage. Interior subdomain tiles use halo mass points; physical lower and
upper boundaries follow each routine's exact copy or duplicate-average
expression. The `calc_mu_uv` variants cover all four periodicity states for
split and already-combined mass. Sixteen randomized `calc_mu_staggered` cases
cross all physical boundary states. The next gates are a randomized big-step
corpus and dependency-closed ARW integration.

Momentum coupling now consumes those staggered masses through the same
backend-owned storage boundary used by future GPU kernels. Every focused output
matches pinned Fortran bits, validation precedes mutation, and one/four-worker
results are identical. The Rust API omits WRF's declared but unused `msfv`
argument; `UPSTREAM_FINDINGS.md` records that interface drift.

Omega diagnosis now completes the fourth `rk_step_prep` diagnostic stage. Its
region enforces the full vertical column assumed by WRF's literal `k=1` and
`k=2` accesses, and it validates horizontal C-grid neighbors before mutation.
All focused values match pinned Fortran, including exceptional classes, and
one/four-worker outputs are identical. The Rust API omits four map-factor
arrays that the source routine never reads.

Moisture coefficients now complete the fifth `rk_step_prep` diagnostic stage.
The region derives the distinct `cqu`, `cqv`, and `cqw` clipping rules and
requires WRF's west/south halo neighbors before mutation when species are
active. `MoistureSpecies`
contains only Registry slots 2 through `n_moist`; an empty view selects the dry
`1, 1, 0` defaults. All 8,232 focused output and sentinel values match pinned
Fortran, and one/four-worker results are identical.

Full inverse density now completes the sixth `rk_step_prep` diagnostic stage.
`InverseDensityRegion` clips every upper stagger to the mass grid and validates
all field shapes before mutation. All 2,352 focused output and sentinel values
match pinned Fortran, including exceptional classes, and one/four-worker
results are identical. Inactive storage is preserved explicitly rather than
depending on WRF's partial `INTENT(OUT)` definition.

Pressure-point geopotential now completes the seventh and final diagnostic call
currently made by `rk_step_prep`. `PressurePointGeopotentialRegion` validates
the full level read at `k+1`, clips all upper staggers, and checks field roles
before mutation. All 2,352 focused output and sentinel values match pinned
Fortran, including a reassociation-sensitive overflow case, and one/four-worker
results are identical.

Dry large-timestep tendency assembly now ports `rk_addtend_dry` immediately
after diagnostic preparation. Typed borrowed groups separate mutable RK
tendencies, persistent forward/physics tendencies, saved boundary tendencies,
thermodynamics, and the four map factors the source actually reads. The phase
enum distinguishes first-substep accumulation from later reuse. Component
ranges preserve the distinct U, V, W/geopotential, temperature, and mass
stagger rules, including interior `kte-1` versus full-stagger `kte` behavior.
All 5,616 focused values match the extracted pinned Fortran routine; validation
is failure-atomic and one/four-worker results are bit-identical.

Acoustic small-step preparation now ports `small_step_prep` at the next
trajectory boundary. First substeps replace previous time levels and reset the
mass perturbation; later substeps construct reference differences and
west/south mass averages. Both paths form the source-ordered U, V, W,
temperature, geopotential, pressure-coefficient, saved-state, and omega fields
over their exact stagger ranges. All 9,936 focused values match the extracted
pinned Fortran routine; validation is failure-atomic and one/four-worker
results are bit-identical. The Rust API omits thirteen unused source arguments
and only dead full-column `ww_save` stores.

Acoustic pressure diagnosis now ports `calc_p_rho`, which is called before and
during every acoustic loop. The nonhydrostatic path diagnoses inverse density
from the geopotential gradient and applies the linearized equation of state.
The hydrostatic path diagnoses pressure/inverse density and integrates
geopotential upward. A separate phase enum preserves initialization versus
forward pressure-history damping. All 3,456 focused values match the extracted
pinned Fortran routine; validation is failure-atomic and one/four-worker
results are bit-identical. The Rust API omits nine unused source arguments.

Vertical acoustic coefficient construction now ports `calc_coef_w`, which
factorizes the tridiagonal system consumed by the implicit `advance_w` solve.
The region makes WRF's complete-column behavior explicit instead of accepting
the ignored vertical tile bounds. Typed output roles distinguish the lower
diagonal, reciprocal eliminated diagonal, and upper elimination factor. All
3,024 focused values match the extracted pinned Fortran routine; validation is
failure-atomic and one/four-worker results are bit-identical. The Rust API
omits the unused coefficient arrays and ignored vertical tile inputs, and the
implementation removes the redundant horizontal scalar array.

Acoustic flux accumulation now ports `sumflux`, the stage between vertical
momentum and the closing pressure diagnosis. The region derives distinct U, V,
and W stagger ranges, while the first substep preserves WRF's full-tile zeroing
for every output. Finalization divides each nonlinear sum by the substep count
and restores the saved linear flux in source operation order. All 375 stored
bits match the extracted routine after three substeps; invalid phases, field
shapes, and coefficient lengths fail before mutation. The Rust API omits ten
unused source arguments and allocates no numerical scratch.

### `wrf-io`

Implemented:

- a typed minimum ARW initialization/restart schema with checked mass and
  staggered dimensions, fixed WRF timestamps, ordered metadata, and 13 core
  variables;
- current v4.7.1 Registry data naming, including `THM` rather than historical
  `T`;
- borrowed complete-dataset validation before filesystem mutation;
- safe classic NetCDF 64-bit-offset output without field-sized clones;
- NetCDF-3/4 input through the safe GeoRust wrapper into caller-owned buffers;
- exact ordered restart schema, metadata, and raw-bit comparison with two
  reusable scratch buffers capped at one MiB each; and
- an independent NetCDF-C oracle that matches the Rust fixture exactly.

The current reader intentionally accepts only dimensions and primitive types in
the minimum schema. Full restart support still needs all Registry-selected
fields and dimensions, WRF alarm attributes, multiple records, NetCDF-4 output
policy, and a resumed idealized trajectory.

Matched classic-format I/O measurements put 1,000 tiny complete restart writes
in the same practical class (0.56 seconds NetCDF-C, 0.44 seconds Rust). For 400
MiB of bulk field overwrites, Rust took 0.543888 seconds versus 0.242086 for
NetCDF-C, while separate peak RSS was 19.6 MB versus 28.6 MB. A one-MiB standard
buffer removed an initial per-value syscall pathology. The remaining 2.25×
bulk gap is tracked; do not add a bespoke unsafe or SIMD serializer without an
end-to-end profile and exact parity evidence.

## Performance decisions

- Release profile: thin LTO, one codegen unit.
- Allocate fields and scratch buffers during setup, not timesteps.
- Prefer contiguous structure-of-arrays field layouts.
- Preserve WRF precision and operation order until parity is established.
- `pulp` 0.22.3 is the accepted stable-Rust SIMD layer for Held-Suarez damping
  because it supports one runtime ISA dispatch per kernel. It remains subject
  to per-kernel evidence; `wide` is still a controlled-target candidate.
- `std::simd` is not the stable production baseline while it requires nightly.
- SIMD dispatch happens once per kernel and runs inside CPU worker chunks.
- Criterion 0.7 is a dev-only statistical benchmark dependency; 0.8 is excluded
  because it exceeds the workspace's declared Rust 1.85 minimum.
- Matched Fortran performance is a stopping signal as well as an optimization
  signal. If portable Rust is already close, the standard multithreaded path is
  competitive, allocations are bounded, and no full-model profile shows a
  material hotspot, record the result and move on. Do not spend port time on a
  fragile marginal win.

See `docs/architecture/compute_backends.md`,
`docs/architecture/performance_principles.md`, and
`docs/architecture/simd.md`.

## Positive-definite performance baseline

The 1,048,576-value Criterion baseline on an Apple M3 Max measured:

- sheet: 1.1569 ms (1 worker), 316.00 µs (4), 274.85 µs (16);
- slab: 1.8084 ms (1 worker), 483.08 µs (4), 347.98 µs (16).

Four workers capture most sheet scaling; 16 workers reach 4.21× sheet and 5.20×
slab speedup over one. AArch64 release assembly contains scalar rather than
packed NEON arithmetic in the pointwise loops. See
`docs/performance/positive-definite-2026-07-13.md` for environment, confidence
intervals, throughput, and caveats. End-to-end steady-state allocation
measurement shows one or two 1,520-byte Rayon/crossbeam queue allocations per
100 dispatches, independent of worker count, and no reallocations. Do not claim
zero Rayon-dispatch allocations; the precise measured amortized cost is
0.01–0.02 allocations and 15.2–30.4 bytes per million-point call.

`pulp` 0.22.3 was then prototyped for only the pointwise passes. It preserved
exact bits across 14 vector/tail lengths and the upstream fixtures, but slowed
the one- and four-worker benchmarks by about 1–4%. Gains appeared only at 16
workers. The implementation and dependency were removed; keep the scalar path
for this kernel. `pulp` remains a candidate for more pointwise-dominant kernels.

Matched GNU Fortran 14.2.0 `-O3 -flto` medians are 1.709219 ms for
`positive_definite_sheet` and 2.336656 ms for `positive_definite_slab` on the
same 1,048,576-value all-correction workload. One-worker Rust is respectively
1.48× and 1.29× faster; 16-worker Rust is 6.22× and 6.71× faster than serial
Fortran. These are combined routine results, not isolated attribution of scratch
copies versus the repeated negativity scan.

## Held-Suarez performance baseline

For 2,097,152 momentum updates on the Apple M3 Max, accepted safe SIMD measured
0.93459 ms with one Rust worker, 0.29105 ms with four, and 0.52122 ms with 16. A
matched GNU Fortran 14.2.0 `-O3 -flto` run of the pinned routine measured a
0.859712 ms median across seven samples. Current Rust is 8.7% slower serially and 2.95×
faster with four workers than serial Fortran. This is an isolated-kernel result,
not a whole-model claim. See `PERFORMANCE_PARITY.md` and
`docs/performance/held-suarez-2026-07-13.md`.

The `pulp` implementation preserves exact scalar bits for every tested line
length from 1 through 257 and improves the scalar baseline by 4.4–5.4% across
worker counts. Its warmed two-pass dispatch uses three 1,520-byte scheduler
allocations per 100 calls, with no reallocations or numerical scratch.

Bench-only native CPU and/or fat-LTO builds were screened. Native+fat gained
1.9% on serial Held-Suarez but regressed its four- and 16-worker cases. It was
flat to slightly slower for positive-definite at one and four workers, with
gains limited to the noisier 16-worker cases. The separated settings also failed
to improve representative worker counts consistently. Keep the portable
ThinLTO production/benchmark baseline.

## Column-mass staggering performance baseline

For 2,099,200 momentum-mass outputs on a 1,024 × 1,024 physical domain, scalar
Rust measured 332.80 µs with one worker, 115.32 µs with four, and 242.03 µs with
16. Matched GNU Fortran 14.2.0 `-O3 -flto` measured a 286.850 µs median across
eleven samples. Rust is 16.0% slower serially and 2.49× faster with four workers
than serial Fortran. These are isolated-routine results.

Each warmed 100-dispatch phase at 1, 4, and 16 workers recorded three 1,520-byte
scheduler allocations, or 4,560 bytes total, with no reallocations and no
numerical scratch. A parity-correct `pulp` prototype regressed representative
one- and four-worker results and was removed. A safe slice-iterator formulation
also regressed serial performance and was removed. Keep the readable scalar
implementation. See `docs/performance/column-mass-staggering-2026-07-13.md`.

The doubly periodic big-step variant measured 359.64 µs with one Rust worker,
181.10 µs with four, and 400.40 µs with 16. Matched GNU Fortran 16.1.0
`-O3 -flto` measured a 347.120 µs median. Serial Rust is within 3.6%; four
workers are 1.92× faster than serial Fortran. Its warmed allocation profile is
the same three 1,520-byte scheduler allocations per 100 calls, with no
reallocations or numerical scratch. This is close enough to stop tuning and
retain the readable scalar implementation. See
`docs/performance/periodic-column-mass-2026-07-13.md`.

## Momentum-coupling performance baseline

For 7,950,336 outputs on a 256 × 256 × 40 C-grid workload, accepted safe Rust
measured 1.3679 ms with one worker, 654.95 µs with four, and 1.4425 ms with 16.
Matched GNU Fortran 16.1.0 `-O3 -flto` measured a 1.152625 ms median. Rust is
18.7% slower serially and 1.76× faster with four workers than serial Fortran.

Replacing repeated global indexing with validated equal-length row slices
preserved all oracle bits and improved representative timings by about 77%.
Every warmed 100-call phase recorded five 1,520-byte scheduler allocations and
no reallocations or numerical scratch. The result is close enough to stop
tuning without explicit SIMD. See
`docs/performance/momentum-coupling-2026-07-13.md`.

## Omega-diagnosis performance baseline

For 2,686,976 outputs on a 256 × 256 × 40 complete-column workload, accepted
safe Rust measured 5.0201 ms with one worker, 1.3306 ms with four, and 666.90 µs
with 16. Matched GNU Fortran 16.1.0 `-O3 -flto` measured a 1.832250 ms median.
Rust is 2.74× slower serially, 1.38× faster with four workers, and 2.75× faster
with 16 than serial Fortran.

Replacing the first parity-correct column-strided traversal with typed,
equal-length west-east row views preserved every oracle value and improved
representative timings by about 70%. Settled 100-call phases recorded one
1,520-byte scheduler allocation and no numerical scratch. The standard
multithreaded path is faster than Fortran, so no explicit SIMD is being added.
See `docs/performance/omega-diagnosis-2026-07-13.md`.

## Moisture-coefficient performance baseline

For 7,819,264 outputs on a 256 × 256 × 40 workload with six active species,
safe Rust measured 7.1239 ms with one worker, 2.0418 ms with four, and 3.4246 ms
with 16. Matched GNU Fortran 16.1.0 `-O3 -flto` measured a 5.221150 ms median.
Rust is 36.4% slower serially, 2.56× faster with four workers, and 1.52× faster
with 16 than serial Fortran.

Each output row temporarily holds the ordered species total before being
overwritten by its final coefficient, removing WRF's automatic `qtot` scratch.
Every first and settled 100-call phase recorded five 1,520-byte scheduler
allocations, no reallocations, and no numerical scratch. The standard
multithreaded path is already faster than Fortran, so explicit SIMD and custom
worker selection are not justified. See
`docs/performance/moisture-coefficients-2026-07-13.md`.

## Full inverse-density performance baseline

For 2,621,440 outputs on a 256 × 256 × 40 mass-grid workload, safe Rust measured
325.94 µs with one worker, 121.02 µs with four, and 397.32 µs with 16. Matched
GNU Fortran 16.1.0 `-O3 -flto` measured a 210.880 µs median. Rust is 54.6%
slower serially and 1.74× faster with four workers than serial Fortran.

The safe hot loop adds equal-length contiguous slices without numerical
scratch. First and settled 100-call phases recorded at most two and one
1,520-byte scheduler allocations respectively, with no reallocations. The
standard four-worker path is competitive, so explicit SIMD and custom
scheduling are not justified. See
`docs/performance/inverse-density-2026-07-14.md`.

## Pressure-point geopotential performance baseline

For 2,621,440 outputs on a 256 × 256 × 40 mass-grid workload, safe Rust measured
444.82 µs with one worker, 140.72 µs with four, and 408.52 µs with 16. Matched
GNU Fortran 16.1.0 `-O3 -flto` measured a 402.140 µs median. Rust is 10.6%
slower serially and 2.86× faster with four workers than serial Fortran.

The safe hot loop reads four equal-length contiguous slices and preserves WRF's
addition order without numerical scratch. First and settled 100-call phases
recorded at most two and one 1,520-byte scheduler allocations respectively,
with no reallocations. The result is close enough to stop tuning without
explicit SIMD. See
`docs/performance/pressure-point-geopotential-2026-07-14.md`.

## Integrated Runge-Kutta preparation performance baseline

On a matched 256 × 256 × 40 grid with two active moisture species, serial GNU
Fortran 16.1.0 `-O3 -flto` measured a 6.0671 ms median. Release Rust measured
10.092 ms with one worker, 3.3025 ms with four, and 4.5476 ms with 16. Four
workers are 1.84× faster than serial Fortran; the default 16-worker path is
1.33× faster.

Every settled 100-call phase records 19 scheduler allocations totaling 28,880
bytes, no reallocations, no numerical scratch, and no field clones. The normal
parallel path clears the performance gate, so cross-stage fusion and additional
SIMD wait for a coupled trajectory profile. See
`docs/performance/runge-kutta-preparation-2026-07-14.md`.

## Dry tendency assembly performance baseline

On a matched 256 × 256 × 40 first-substep workload, serial GNU Fortran 16.1.0
`-O3 -flto` measured an 8.425600 ms median. Portable release Rust measured
18.625 ms with one worker, 4.9522 ms with four, and 2.5235 ms with 16. Four and
16 workers are 1.70× and 3.34× faster than serial Fortran.

Every settled 100-call phase records nine scheduler allocations totaling
13,680 bytes, no reallocations, no numerical scratch, and no field clones. A
FatLTO screen was statistically flat. Parallel execution clears the gate, so
explicit SIMD and further fusion wait for a trajectory profile. See
`docs/performance/dry-tendency-assembly-2026-07-14.md`.

## Acoustic small-step preparation performance baseline

On a matched 256 × 256 × 40 first-substep workload, serial GNU Fortran 16.1.0
`-O3 -flto` measured a 5.877800 ms median. Portable release Rust measured
26.123 ms with one worker, 7.4138 ms with four, and 6.5595 ms with 16. The
ordinary parallel paths are 26.1% and 11.6% slower than serial Fortran,
respectively, which is within the project's operational stopping range.

Every settled 100-call phase records 28 scheduler allocations totaling 42,560
bytes, no reallocations, no numerical scratch, and no field clones. Explicit
SIMD and more complex pass fusion wait for a coupled trajectory profile. See
`docs/performance/acoustic-step-preparation-2026-07-14.md`.

## Acoustic pressure diagnosis performance baseline

On matched 256 × 256 × 40 workloads, serial GNU Fortran 16.1.0 `-O3 -flto`
measured 1.529500 ms nonhydrostatic and 1.602750 ms hydrostatic. Portable
release Rust with four workers measured 0.81126 and 0.95950 ms, respectively:
1.89× and 1.67× faster than serial Fortran. One-worker Rust was 19.8% and 29.9%
slower.

Changing the safe hydrostatic recurrence from column-strided to WRF-like
level-major traversal improved serial Rust from 8.63 to 2.08 ms while retaining
every oracle bit. Settled 100-call phases record at most five small scheduler
allocations, no reallocations, no numerical scratch, and no field clones. The
ordinary four-worker path clears the gate, so explicit SIMD and custom worker
selection wait for a trajectory profile. See
`docs/performance/acoustic-pressure-2026-07-14.md`.

## Vertical acoustic coefficient performance baseline

On a matched 256 × 256 × 40 complete-column workload, serial GNU Fortran
16.1.0 `-O3 -flto` measured a 1.867500 ms median. Portable release Rust
measured 14.608 ms with one worker, 3.8912 ms with four, and 1.7109 ms with 16.
The default 16-worker host path is 1.09× faster than serial Fortran.

Changing the first exact implementation from column-strided to WRF-like
level-major contiguous-X traversal improved serial Rust from 27.881 to 14.608
ms without changing any oracle bit. Every settled 100-call phase records three
small scheduler allocations totaling 4,560 bytes, no reallocations, no
numerical scratch, and no field clones. The default path clears the gate, so
the remaining serial vectorization gap waits for an integrated trajectory
profile. See
`docs/performance/vertical-acoustic-coefficients-2026-07-14.md`.

## Acoustic flux accumulation performance baseline

On a matched three-substep 256 × 256 × 40 workload, serial GNU Fortran 16.1.0
`-O3 -flto -ffp-contract=off` measured a 5.513750 ms median. Portable release
Rust measured 26.048 ms with one worker, 7.2084 ms with four, and 3.6192 ms
with 16. The standard 16-worker path is 1.52× faster than serial Fortran.

Every settled 100-sequence phase records 19 scheduler allocations totaling
28,880 bytes, no reallocations, no numerical scratch, and no field clones.
Explicit SIMD and additional pass fusion stop pending the coupled acoustic
trajectory profile. See
`docs/performance/acoustic-flux-accumulation-2026-07-14.md`.

## Kessler microphysics performance baseline

For 655,360 grid points on a 128 × 128 × 40 domain, one-worker Rust measured
30.944 ms and matched GNU Fortran 14.2.0 `-O3 -flto` measured a 31.7804 ms
median. Four-worker Rust measured 8.9176 ms and 16-worker Rust measured 5.0144
ms, respectively 3.56× and 6.34× faster than serial Fortran. Serial performance
is already within 2.6%, so no SIMD or more complex layout is being pursued.

The reusable workspace uses 2,621,600 bytes with one worker and 2,624,000 bytes
with 16 workers on this workload. Every settled 100-call measurement recorded
three 1,520-byte scheduler allocations and no reallocations; numerical scratch
was allocated only at workspace creation. See
`docs/performance/kessler-microphysics-2026-07-13.md`.

## Seeded randomized ARW parity

`tools/arw-corpus-generator` produces committed language-neutral inputs for all
translated dynamics routines. The corpus contains 24 sheet cases, 16 slab
cases, 12 Held-Suarez cases, and 16 column-mass cases. It varies small shapes,
negative and non-one memory origins, domain/tile clipping, signed zero, finite
magnitude extremes, and active NaN/infinity values. The column-mass cases cover
all 16 cross-axis physical-boundary combinations.

`scripts/randomized-arw/run-oracles.sh` regenerates and byte-compares the input
files before compiling the pinned WRF routines. Rust consumes those same inputs
and all 39,588 Fortran-derived output records. Finite values, signed zero, and
infinities match raw bits; NaN matches by class because payload propagation is
not portable. Current default-host-parallel Rust passes every case.

Finite extreme sheet seed `1695930` and slab seed `2771003` reproduce
intermediate multiplication overflow in WRF's normalization. Rust preserves the
infinity results; `UPSTREAM_FINDINGS.md` records this as WRF-008 rather than
silently changing operation order.

## WRF time oracle

The bundled Fortran `external/esmf_time_f90/Test1.F90` is compiled locally
with Homebrew `gfortran` and its output matched `Test1.out.correct` exactly.

Two build details are required:

1. preprocess with `TIME_F90_ONLY` and compile generated `.f` files as free
   form;
2. replace `defaultCalendar=` with the implementation's actual
   `defaultcalkind=` keyword in the generated test copy only.

The second item is an upstream v4.7.1 test/interface mismatch. Never patch the
pinned upstream source to hide it. `scripts/run-wrf-time-oracle.sh` reproduces
the complete sequence without modifying the pinned source.

The upstream golden output contains 89 named `PASS` arithmetic/formatting cases
plus four clock cases, for 93 active cases total. All 93 names now occur in
executing Rust assertions, verified by
`scripts/check-wrf-time-case-coverage.sh`.

## Last verified commands

```text
cargo fmt --all
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
cargo test --workspace --release
./scripts/check-wrf-time-case-coverage.sh
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
./scripts/run-vertical-acoustic-coefficients-oracle.sh
./scripts/run-acoustic-horizontal-momentum-oracle.sh
./scripts/run-acoustic-mass-theta-oracle.sh
./scripts/run-acoustic-vertical-momentum-oracle.sh
./scripts/run-acoustic-flux-accumulation-oracle.sh
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
cargo bench -p wrf-physics --bench kessler_microphysics -- --noplot
cargo run -p wrf-dynamics --release --example measure_held_suarez_allocations
cargo run -p wrf-dynamics --release --example measure_column_mass_staggering_allocations
cargo run -p wrf-dynamics --release --example measure_momentum_coupling_allocations
cargo run -p wrf-dynamics --release --example measure_omega_diagnosis_allocations
cargo run -p wrf-dynamics --release --example measure_moisture_coefficient_allocations
cargo run -p wrf-dynamics --release --example measure_inverse_density_allocations
cargo run -p wrf-dynamics --release --example measure_pressure_point_geopotential_allocations
cargo run -p wrf-dynamics --release --example measure_runge_kutta_preparation_allocations
cargo run -p wrf-dynamics --release --example measure_dry_tendency_assembly_allocations
cargo run -p wrf-dynamics --release --example measure_acoustic_step_preparation_allocations
cargo run -p wrf-dynamics --release --example measure_acoustic_pressure_allocations
cargo run -p wrf-dynamics --release --example measure_vertical_acoustic_coefficient_allocations
cargo run -p wrf-dynamics --release --example measure_acoustic_flux_accumulation_allocations
cargo run -p wrf-physics --release --example measure_kessler_allocations
```

Result: 200 unit tests and 16 doctests passed in debug and release modes (one
corpus-generator test, 15 `wrf-compute`, 15 `wrf-domain`, two
`wrf-domain-mpi`, 113 `wrf-dynamics`, eight `wrf-physics`, 15 `wrf-registry`,
11 `wrf-io`, and 20 `wrf-time`),
including all-target benchmark smoke execution. Clippy and rustdoc are clean.
All 93 active WRF time cases are referenced by executing Rust assertions, both
Fortran time interfaces match `Test1.out.correct`, the focused numerical
oracles remain exact, and all four randomized Fortran corpora pass their 39,588
complete-output comparisons. The column-mass matched benchmark,
one/four/host-worker Criterion run, allocation budget, optimized assembly
inspection, and rejected SIMD screen remain recorded in the performance ledger
and detailed baseline. Both big-step column-mass entry points also match all
960 focused periodic/physical oracle values; their matched timing and
allocation evidence are recorded. Momentum coupling matches all 3,780 focused
values; its matched timing, safe hot-loop correction, and allocation evidence
are recorded. Omega diagnosis matches all 1,960 focused values; its
complete-column contract, row-layout correction, matched timing, and allocation
evidence are recorded. Moisture coefficients match all 8,232 focused values;
their generated-species boundary, three stagger ranges, matched timing, and
allocation evidence are recorded. Full inverse density matches all 2,352
focused values; its three-axis clipping, matched timing, and allocation evidence
are recorded. Pressure-point geopotential matches all 2,352 focused values; its
upper-full-level contract, source operation order, matched timing, and
allocation evidence are recorded. Integrated Runge-Kutta preparation matches
all 1,728 coupled intermediate, final, and sentinel values; its all-stage
preflight, matched timing, and allocation evidence are recorded. The WRF
dry-tendency oracle matches all 5,616 first/later-substep, stagger, persistent,
and sentinel values; its matched timing and allocation evidence are recorded.
Acoustic small-step preparation matches all 9,936 first/later-substep,
staggered, full-column, exceptional, and sentinel values; its matched timing,
allocation evidence, and performance stopping decision are recorded.
Acoustic pressure diagnosis matches all 3,456 mode/phase, partial-tile,
hydrostatic-recurrence, exceptional, and sentinel values; its matched timing,
allocation evidence, layout correction, and performance stopping decision are
recorded.
Vertical acoustic coefficient construction matches all 3,024 boundary-mode,
partial-tile, complete-column, exceptional, and sentinel values; its matched
timing, allocation evidence, layout correction, and deferred vectorization
opportunity are recorded.
Horizontal acoustic momentum implements `advance_uv` in nested boundary,
domain, field, and CPU modules. A direct rigid-lid nonhydrostatic oracle matches
all 432 stored U/V values exactly; branch, failure-atomicity, and worker tests
pass. On the matched 256 × 256 × 40 workload, 16-worker Rust is 3.1% slower
than optimized serial Fortran, so further SIMD tuning is deferred.
Acoustic mass, omega, and theta advancement implements `advance_mu_t` with a
complete-column typed region and scratch-free output reuse. Four direct global,
nested, periodic-X, and partial-horizontal cases match all 3,168 stored values
exactly. On the matched 256 × 256 × 40 workload, 16-worker Rust is 1.27× faster
than optimized serial Fortran; SIMD tuning stops.
Implicit acoustic vertical advancement implements `advance_w` in nested
boundary, field, region, and CPU modules. Four direct cases cover both
`phi_adv_z` formulations, rigid/nonrigid tops, upper damping, terrain
reconstruction, lateral policies, and partial horizontal tiles; all 2,592
state values match exactly. The complete-column contract and reusable 10.67 MiB
RHS workspace are explicit. On the matched workload, four-worker Rust is 4.1%
faster and 16-worker Rust is 2.53× faster than optimized serial Fortran, so
SIMD tuning stops.
Acoustic flux accumulation implements `sumflux` in nested field, region, and
CPU modules. A direct three-substep oracle matches all 375 complete-storage
bits, including first-step zeros at stagger-only tile points. On the matched
workload, 16-worker Rust is 1.52× faster than optimized serial Fortran, uses no
numerical scratch, and records 28,880 bytes of scheduler allocation per 100
settled sequences, so further SIMD tuning stops.
The WRF
Registry oracle matches five generated includes
and eight state-metadata records exactly. Domain decomposition and clipped
tiles match pinned WRF routines, periodic destinations match `period.c`
exactly, and complete four-rank MPI patch memory matches the local executor.
The Kessler oracle matches all 660 mutable values exactly; its matched timing,
one/four/host-worker scaling, and explicit workspace/allocation accounting are
recorded in the performance ledgers.
The independent NetCDF-C and Rust minimum restart files are byte-identical and
also pass typed schema, metadata, and raw-bit comparison.

## Maintained knowledge and quality ledgers

- `docs/wiki/README.md`: technical encyclopedia and onboarding index.
- `TEST_COVERAGE.md`: upstream, adversarial, concurrency, and operational gaps.
- `UPSTREAM_FINDINGS.md`: reproducible Fortran bugs, test gaps, and performance
  opportunities with confidence labels.
- `PERFORMANCE_PARITY.md`: matched Rust/Fortran workload policy and cumulative
  speed ratios.
- Public crates enable missing-doc warnings and deny broken rustdoc links.

## Git checkpoints

- `bb6cc55` — pinned source tooling, time parity, compute architecture, both
  positive-definite kernels, wiki, coverage ledger, and upstream findings.
- `0ee002d` — Criterion throughput/scaling harness and scalar baseline.
- `7389443` — instrumented steady-state allocation budgets and measurements.
- `5b5f7aa` — documented and removed a parity-correct SIMD prototype that
  regressed representative positive-definite benchmarks.
- `4e6af9a` — nested scientific module hierarchy, Held-Suarez scalar parity,
  matched optimized-Fortran benchmark, wiki, and coverage/findings updates.
- `58bcb67` — accepted safe Held-Suarez SIMD, allocation evidence, scalar/SIMD
  parity corpus, and docs.rs example.
- `d0ec31d` — matched positive-definite Fortran benchmark, optimization-level
  audit, and rejected bench-only native/fat-LTO profiles.
- `8d5e112` — durable state pointer for the benchmark checkpoint.
- `adef46f` — interior ARW column-mass staggering, exact extracted-source
  oracle, typed ranges/errors, concurrency coverage, wiki, and findings.
- `67d9ce3` — all `calc_mu_staggered` physical-boundary paths, domain/tile/memory
  separation, 240-value exact Fortran corpus, and all-boundary determinism.
- `dd3e903` — matched column-mass Criterion/Fortran benchmark harnesses and
  warmed allocation instrumentation.
- `f6dd8e6` — versioned seeded ARW input generator, four pinned Fortran corpus
  drivers, 39,588 complete-output comparisons, CI gate, and exceptional-value
  policy in rustdoc.
- `dcb30e3` — typed Registry parser, selected exact artifact generator, upstream
  fixture/goldens, malformed-input coverage, and CI oracle.
- `076caa1` — Registry architecture, language inventory, wiki, state ledgers,
  and confirmed upstream allocation-generator finding.

## Immediate next actions

1. Compare a complete acoustic small-step trajectory through preparation,
   pressure diagnosis, coefficient construction, horizontal momentum,
   mass/theta, vertical momentum, flux accumulation, and closing pressure
   diagnosis.
2. Extend NetCDF/restart support to arbitrary Registry-selected dimensions and
   fields, WRF alarm metadata, and a resumed idealized trajectory.
3. Add Registry-generated asymmetric halo descriptors and multi-field message
   aggregation to the domain transport.
4. Extend Registry preprocessing with includes and conditional definitions.
5. Add Registry-backed state binding and a short prognostic trajectory through
   integrated `rk_step_prep`, `rk_addtend_dry`, acoustic preparation, and
   pressure diagnosis.
6. Add Registry packages, typedefs, communication entries, and remaining
   generated artifacts in dependency-closed slices.
7. Measure Held-Suarez SIMD on x86-64 when that architecture is available.
8. Connect Kessler fields to Registry moisture species and the microphysics
   driver, then add a coupled precipitation trajectory.
