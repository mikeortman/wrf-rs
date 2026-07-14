# Dry large-step boundary-stage performance

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64.

## Matched workload

Both programs execute the pinned WRF v4.7.1 first-substep sequence
`relax_bdy_dry`, `rk_addtend_dry`, `spec_bdy_dry` on a nested, nonperiodic
256 × 256 × 40 physical mass grid. Storage includes the same upper stagger and
halos. Boundary width is five with a one-point specified zone and a four-point
relaxation zone, WRF's default lateral-boundary configuration. Fields, boundary
slabs, and the tile-halo mass-weighting workspace are allocated and initialized
once, outside timing.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses the workspace
bench profile: optimization level 3, ThinLTO, one codegen unit, and no
fast-math or native-CPU flag. These are comparable production optimization
tiers, not identical compiler pipelines.

## Results

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 12.1994 ms median | 1.00× |
| Rust, one worker | 32.950 ms | 0.37× |
| Rust, four workers | 9.1393 ms | 1.33× faster |
| Rust, 16 workers | 5.5310 ms | 2.21× faster |

Fortran's 31 samples span 11.4992–12.8298 ms. Criterion intervals are
32.775–33.135 ms, 9.1083–9.1679 ms, and 5.4968–5.5666 ms for one, four, and
sixteen workers.

Across 100 warmed dispatches at one, four, and sixteen workers, the composed
stage records at most 34 scheduler allocations totaling 51,680 bytes, with no
reallocations, no numerical scratch, and no field clones; the only reusable
buffer is the caller-owned relaxation workspace created during setup.

The composed capability reuses the three independently parity-tested kernels
behind one cross-stage preflight. The serial gap concentrates in the same
scalar loops already profiled for the component kernels. Because the ordinary
four-worker and default sixteen-worker paths are already faster than serial
WRF, explicit SIMD, cross-stage loop fusion, and duplicated specialization are
not justified without an end-to-end trajectory profile.

## Reproduce

```sh
./scripts/benchmark-dry-large-step-boundary-stage-fortran.sh
cargo bench -p wrf-dynamics --bench dry_large_step_boundary_stage -- --noplot
cargo run -p wrf-dynamics --release --example measure_dry_large_step_boundary_stage_allocations
```
