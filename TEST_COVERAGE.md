# Test coverage and gap inventory

This is the test-planning ledger. The root `README.md` tracks implementation
scope; this file tracks confidence within that scope.

## Required layers

Every numerical slice should accumulate five kinds of evidence:

1. **Upstream regression parity** — port every applicable named upstream test.
2. **Differential oracle coverage** — compile the pinned Fortran routine and
   compare identical fixtures with Rust.
3. **Adversarial coverage** — add branch boundaries, malformed shapes, signed
   zero, extrema, and scientifically meaningful invariants absent upstream.
4. **Concurrency determinism** — compare one worker with multiple workers and
   exercise scheduling failures.
5. **Operational coverage** — compare full fields, restarts, I/O metadata, and
   trajectories in representative WRF cases.

## Current matrix

| Surface | Upstream parity | Added gap tests | Remaining gaps |
|---|---|---|---|
| `wrf-time` | 93/93 active `Test1.F90` cases; `ESMF_` and `WRFU_` golden outputs exact | year zero, negative year, rational normalization, invalid components | randomized long-clock sequences; leap-second policy when a caller requires it |
| CPU chunk scheduler | Not an upstream surface | disjoint writes, synchronization-confirmed concurrent workers in debug/release, typed kernel error, worker panic | nested-pool behavior; large NUMA systems; cancellation semantics |
| CPU exact-block scheduler | Not an upstream surface | exact boundaries, invalid shapes, worker panic | empty-output contract; scaling and allocation measurements |
| Registry DSL and selected generators | WRF v4.7.1 Registry executable generates five byte-identical include files and an exact eight-record metadata projection from the same fixture | physical source locations; continuations; comments and quote/case behavior; all dimension-length modes; boundary modifiers; time levels; all staggering flags; duplicate/unknown dimensions; malformed counts, ranges, quotes, and unsupported categories | includes and conditionals; typedef/package/communication forms; brace dimensions and four-dimensional scalar arrays; remaining generators; full `Registry.EM` parse |
| Domain decomposition and halo exchange | Pinned `task_for_point.c` assigns all patches exactly for three centered-remainder grids; pinned `region_bounds` plus `set_tiles2` clipping semantics match edge tiles; pinned `period.c` and `f_pack.F90` match every selected periodic destination and doubly periodic corner | signed half-open conversion, non-one origins, WRF guard-point memory bounds, invalid layouts before construction, clipped edge tiles, internal edges/corners, field staggering, bounded message buffers, complete local-vs-four-rank MPI memory equality | generated asymmetric halo descriptors; one-dimensional and larger process grids under MPI; nested/intermediate domains; multi-field message aggregation; operational model fields |
| `positive_definite_sheet` | Pinned Fortran routine compiled directly; 9 focused exact-bit cases plus 24 seeded randomized cases | epsilon boundary, signed zero, finite magnitude extremes, explicit NaN/infinity policy, invalid dimensions/totals, one-vs-four-worker determinism, statistical throughput and warmed allocation budgets | operational field distributions; trajectory integration |
| `positive_definite_slab` | Pinned Fortran routine compiled directly; focused sentinel fixture plus 16 seeded randomized cases | typed half-open region validation, varied non-one memory origins, broader domain/tile clipping, signed zero, finite magnitude extremes, explicit NaN/infinity policy, untouched storage, throughput/scaling and allocation budgets | randomized worker-count determinism; operational field distributions |
| `held_suarez_damp` | Pinned Fortran module compiled directly; 16 focused selections plus 12 seeded randomized complete-field cases | varied non-one memory origins and clipped domains, signed zero, finite magnitude extremes, explicit NaN/infinity policy, staggered-neighbor/range validation, shape mismatch before mutation, one-vs-four-worker determinism, scalar/SIMD raw-bit parity for lengths 1–257, release scaling and allocation baselines | x86-64 SIMD benchmark; complete Held-Suarez trajectory |
| column-mass staggering | Exact `calc_mu_staggered`, `calc_mu_uv`, and `calc_mu_uv_1` bodies extracted from the pinned large module; 240 non-periodic focused values, 960 big-step focused values, and 16 seeded randomized `calc_mu_staggered` complete-field cases | all physical and periodic axis combinations, split and combined mass inputs, overflow-sensitive physical endpoint expressions, missing periodic lower halos, varied non-one memory origins, clipping, signed zero, finite extremes, explicit NaN/infinity policy, validation before mutation, one-vs-four-worker determinism, untouched storage, and matched benchmark/scaling/allocation evidence | randomized big-step corpus; full ARW integration |
| `couple_momentum` | Exact pinned routine body extracted directly; six complete-field cases match all 3,780 west-east, south-north, vertical, and sentinel values by raw bits | independent and combined upper-stagger clipping, negative/non-one Fortran memory origins, half/full levels, finite overflow, zero map factors, all coefficient roles, shape/range/length validation before mutation, one-vs-four-worker determinism, matched benchmark/scaling/allocation evidence | randomized corpus; integrated `rk_step_prep` and trajectory coverage |
| `calc_ww_cp` | Exact pinned routine body extracted directly; five complete-storage cases match all 1,960 omega and sentinel values, with NaNs compared by class | interior and independent/combined upper horizontal boundaries, negative/non-one memory origins, complete physical columns, finite overflow, zero map factors, all field/coefficient roles, every range category, missing-neighbor and incomplete-column validation before mutation, one-vs-four-worker determinism, matched benchmark/scaling/allocation evidence | seeded randomized corpus; integrated `rk_step_prep` and trajectory coverage |
| `calc_cq` | Exact pinned routine body extracted directly; seven complete-storage cases match all 8,232 `cqu`, `cqv`, `cqw`, and sentinel values, with NaNs compared by class | zero, one, and three active species; ignored Registry padding slot; interior and independent/combined upper staggers; non-one/negative memory origins; vertical clipping; finite overflow; signed zero; every range/output/species failure before mutation; one-vs-four-worker determinism; matched benchmark/scaling/allocation evidence | seeded randomized corpus; Registry-generated species binding; integrated `rk_step_prep` and trajectory coverage |
| Kessler microphysics | Pinned `module_mp_kessler.F` compiled directly; all 660 mutable field, halo, and precipitation values match by raw bits | dry/moist columns, four rain regimes, cloud threshold branches, multi-step sedimentation, non-one Fortran horizontal origins, typed parameter/range/shape failures before mutation, one-vs-four-worker determinism, untouched halos, matched benchmark and allocation budget | microphysics driver and Registry species mapping; exceptional-value policy; randomized corpus; coupled precipitation trajectory and restart |
| Minimum NetCDF/restart schema | Independent NetCDF-C and Rust writers produce byte-identical classic 64-bit-offset files plus exactly equal ordered schema, metadata, and field bits for 13 ARW variables | invalid timestamp/dimension/name/spacing; missing, unexpected, duplicate, wrong-type, and wrong-length data; float-attribute raw-bit equality; bounded chunk coverage; exact first differing restart element | complete Registry-selected fields/dimensions; restart alarm state; multiple records; NetCDF-4 write/chunk/compression policy; restart a pinned WRF trajectory from each implementation |

## Fixture policy

Golden files are generated from a named pinned upstream build, never from the
Rust implementation. Floating-point fixtures store raw IEEE-754 bits where
exact parity is expected. A tolerance is allowed only when the responsible
algorithm page documents why exact ordering cannot or should not be retained,
and the test reports absolute, relative, and ULP error separately.

The seeded ARW corpora contain 68 cases and 39,588 complete output values.
Finite values and infinities compare by raw bits. NaN outputs compare by class,
because payload and sign propagation are not portable model contracts. Every
failure reports the seed, field, and first divergent linear index. The oracle
regenerates and byte-compares the committed inputs before compiling Fortran, so
generator drift cannot silently rewrite the test population.

Registry goldens under `parity/registry/golden` are generated by WRF's own
`tools/registry` executable. The Rust unit test compares every byte, while
`scripts/run-registry-oracle.sh` independently regenerates both WRF and Rust
outputs and compares both sides to the committed source-of-truth files.
