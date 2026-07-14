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
- `wrf-dynamics` contains line-parallel, scratch-free ports of WRF's
  positive-definite correction and Held-Suarez momentum damping, checked
  against upstream `REAL` bit patterns.
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
| Registry/configuration | Not started | — | Parse Registry DSL and port generated-state fixtures |
| Domain decomposition / halo exchange | Not started | — | Serial topology first, then MPI differential tests |
| ARW dynamical core | In progress | Positive-definite sheet/slab, Held-Suarez damping, and interior column-mass staggering exact-bit Fortran oracles; CPU scaling baselines for the first two families | Complete column-mass boundary branches and matched benchmark |
| Physics drivers and schemes | Not started | — | Inventory schemes and translate one dependency-closed column |
| I/O and NetCDF metadata | Not started | — | Round-trip WRF files with exact schema parity |
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
./scripts/benchmark-held-suarez-fortran.sh
./scripts/benchmark-positive-definite-fortran.sh
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
