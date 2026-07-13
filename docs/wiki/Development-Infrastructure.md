# Development infrastructure

## Source pinning

`UPSTREAM.toml` records the WRF tag, commit, archive digest, and nested submodule
commits. `scripts/fetch-wrf.sh` reconstructs the reference tree under the
ignored `upstream/WRF` directory. The Rust repository does not silently track a
moving upstream branch; retargeting is an explicit compatibility event.

## Verification gates

The standard checkpoint runs formatting, workspace tests and doctests, Clippy
with warnings denied, Rust documentation with warnings denied, WRF time case
coverage, the full WRF time golden oracle, and each numerical differential
oracle. Numerical slices also need release-mode tests and, before optimization,
representative benchmarks and allocation measurements.

## Documentation contract

Public Rust items explain their invariant, unit, shape/layout, error behavior,
and numerical compatibility implications. Important APIs include compiling
examples. Crates warn on missing public documentation and deny broken rustdoc
links; CI-style verification promotes warnings to failures.

The wiki explains cross-crate concepts and algorithms. Rustdoc explains how to
use a crate safely. `CURRENT_STATE.md` is a compact operational handoff, while
`PORT_STATUS.md`, `TEST_COVERAGE.md`, and `UPSTREAM_FINDINGS.md` are durable
ledgers with distinct purposes.

## Trusted dependencies

Dependencies are selected for maturity, maintenance, portability, and measured
value. Rayon currently provides safe work stealing. SIMD crates are introduced
per kernel, not globally. Future NetCDF, MPI, linear algebra, compression, and
GPU dependencies should wrap established native or Rust ecosystems instead of
reimplementing infrastructure, while keeping unsafe foreign-function details
behind small safe boundaries.
