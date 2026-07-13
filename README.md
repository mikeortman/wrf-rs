# wrf-rs

An incremental, test-driven Rust reimplementation of the Weather Research and
Forecasting (WRF) model.

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
- `wrf-dynamics` contains the first numerical port: an in-place, scratch-free,
  line-parallel implementation of WRF's positive-definite sheet correction,
  checked against upstream `REAL` bit patterns.
- CPU SIMD is selected per translated kernel after scalar parity; see
  `docs/architecture/simd.md`.
- Rust tests cite the corresponding cases in
  `upstream/WRF/external/esmf_time_f90/Test1.F90`.
- [`PORT_STATUS.md`](PORT_STATUS.md) is the source of truth for coverage and
  known gaps.
- [`docs/wiki/README.md`](docs/wiki/README.md) is the technical encyclopedia and
  onboarding map.
- [`UPSTREAM_FINDINGS.md`](UPSTREAM_FINDINGS.md) records reproducible WRF bugs,
  test gaps, and performance opportunities suitable for upstream reporting.
- [`TEST_COVERAGE.md`](TEST_COVERAGE.md) tracks what is tested and what still
  needs adversarial coverage.

## Reproduce

```sh
./scripts/fetch-wrf.sh
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
./scripts/run-wrf-time-oracle.sh
./scripts/run-positive-definite-oracle.sh
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
