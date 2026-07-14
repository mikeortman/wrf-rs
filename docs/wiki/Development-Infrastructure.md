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
oracle. The randomized ARW gate first regenerates and byte-compares its shared
inputs, then runs all four pinned Fortran corpus drivers. Numerical slices also
need release-mode tests and, before optimization, representative benchmarks and
allocation measurements.

Every numerical kernel also receives a matched optimized-Fortran baseline.
Benchmark drivers compile the exact pinned routine, use the same field shape
and active bounds as Rust, exclude setup, and avoid fast-math. Results and any
remaining Rust regression are recorded in `PERFORMANCE_PARITY.md`.

## Documentation contract

Public Rust items explain their invariant, unit, shape/layout, error behavior,
and numerical compatibility implications. Important APIs include compiling
examples. Crates warn on missing public documentation and deny broken rustdoc
links; CI-style verification promotes warnings to failures.

The wiki explains cross-crate concepts and algorithms. Rustdoc explains how to
use a crate safely. `CURRENT_STATE.md` is a compact operational handoff, while
the root `README.md`, `TEST_COVERAGE.md`, and `UPSTREAM_FINDINGS.md` are durable
ledgers with distinct purposes.

## GitHub receipts

Every implementation issue owns one branch and one pull request. The PR closes
the issue, required Rust and Fortran checks gate merge, and GitHub auto-merge
records the successful evidence before deleting the branch.

The canonical wiki Markdown remains under `docs/wiki`. After a pull request
merges, the protected-main verification workflow publishes those pages only
after both Rust and Fortran gates succeed. This gives the GitHub Wiki a separate
generated commit history without publishing unmerged documentation. Use
`scripts/publish-github-wiki.sh` manually only for recovery or an intentional
out-of-band resynchronization.

## Trusted dependencies

Dependencies are selected for maturity, maintenance, portability, and measured
value. Rayon currently provides safe work stealing. SIMD crates are introduced
per kernel, not globally. Future NetCDF, MPI, linear algebra, compression, and
GPU dependencies should wrap established native or Rust ecosystems instead of
reimplementing infrastructure, while keeping unsafe foreign-function details
behind small safe boundaries.
