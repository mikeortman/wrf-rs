# WRF Rust port status

Target: WRF v4.7.1, commit
`f52c197ed39d12e087d02c50f412d90d418f6186`.

The percentages below describe translated interfaces/tests, not scientific
accuracy. Whole-model parity remains 0% until a Rust executable can consume a
WRF initialization and match an upstream integration.

| Area | State | Evidence | Next gate |
|---|---|---|---|
| Source pin and license | Complete | `UPSTREAM.toml`, `scripts/fetch-wrf.sh` | Monitor upstream only by explicit retargeting |
| ESMF-derived time/calendar | Complete for active Test1 surface | 93/93 active cases; both Fortran interfaces match the golden output | Add cases when later WRF callers expose untested behavior |
| Registry/configuration | Not started | — | Parse Registry DSL and port generated-state fixtures |
| Domain decomposition / halo exchange | Not started | — | Serial topology first, then MPI differential tests |
| ARW dynamical core | In progress | positive-definite sheet/slab and Held-Suarez damping exact-bit Fortran oracles; CPU scaling baselines | Extend differential corpora, then select the next dependency-closed kernel |
| Physics drivers and schemes | Not started | — | Inventory schemes and translate one dependency-closed column |
| I/O and NetCDF metadata | Not started | — | Round-trip WRF files with exact schema parity |
| WRFDA, WRF-Chem, WRF-Hydro, TL/adjoint | Not started | — | Separate workstreams after ARW baseline |
| End-to-end idealized/regression suite | Not started | — | `em_b_wave`/`em_squall2d_x` differential runs |

## Definition of parity

- **Discrete parity:** exact values, dimensions, metadata, status codes, and
  emitted configuration.
- **Kernel parity:** per-routine fixtures over edge cases, with exact comparison
  where possible and a checked absolute/relative/ULP policy otherwise.
- **Trajectory parity:** per-timestep norms and selected field comparisons,
  including restart equivalence.
- **Operational parity:** serial and distributed runs, nesting, supported I/O,
  failure behavior, and reproducible reference cases.

Passing a looser end-state tolerance does not excuse an unexplained divergent
trajectory.
