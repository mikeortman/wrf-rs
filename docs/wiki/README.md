# wrf-rs technical wiki

This wiki is the durable conceptual map for both contributors and future agent
sessions. It explains what the code does, why the architecture exists, how it
maps to WRF, and where numerical assumptions are verified. Pages describe the
implemented system; proposed work is explicitly labeled.

## Start here

- [System overview](System-Overview.md) — major WRF subsystems and the Rust
  workspace map.
- [Execution and storage](Execution-and-Storage.md) — memory order,
  multithreading, capability traits, and the future GPU boundary.
- [Parity and testing](Parity-and-Testing.md) — what “same output” means and how
  Fortran oracles prevent self-confirming tests.
- [Timekeeping](Timekeeping.md) — exact rational model time and Gregorian
  calendar behavior.
- [Positive-definite correction](Positive-Definite-Correction.md) — derivation,
  branch semantics, layout, and performance characteristics of the first
  numerical kernel.
- [Development infrastructure](Development-Infrastructure.md) — source pinning,
  scripts, verification gates, and documentation policy.
- [Positive-definite performance baseline](../performance/positive-definite-2026-07-13.md)
  — release throughput, scaling, generated-code findings, and caveats.

## Maintenance rule

Each completed port slice updates its algorithm page, parity evidence,
performance notes, crate-level Rust documentation, `PORT_STATUS.md`,
`TEST_COVERAGE.md`, and `CURRENT_STATE.md`. Findings in the Fortran source also
go into `UPSTREAM_FINDINGS.md` with a confidence label and reproduction.
