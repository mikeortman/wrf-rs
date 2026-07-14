# Coupled dry tendency and boundary-stage performance

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64.

## Matched workload

Both programs execute pinned WRF v4.7.1 `rk_addtend_dry` immediately followed
by nested, nonperiodic `spec_bdy_dry` on a 256 × 256 × 40 physical mass grid.
Storage includes the same upper stagger and halos. Boundary width is three and
specified-zone width is two. Fields and boundary slabs are allocated and
initialized once, outside timing.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses the workspace
bench profile: optimization level 3, ThinLTO, one codegen unit, and no
fast-math or native-CPU flag. These are comparable production optimization
tiers, not identical compiler pipelines.

## Results

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 9.029150 ms median | 1.00× |
| Rust, one worker | 19.995 ms | 0.45× |
| Rust, four workers | 5.8323 ms | 1.55× faster |
| Rust, 16 workers | 3.9960 ms | 2.26× faster |

Fortran's eleven samples span 8.894750–9.238400 ms. Criterion intervals are
19.927–20.068 ms, 5.7993–5.8659 ms, and 3.9553–4.0391 ms for one, four, and
sixteen workers.

The coupled Rust capability adds no numerical scratch or field clone. It
reuses the independently parity-tested kernels and performs one cross-stage
preflight. Because the normal multithreaded configurations are already faster
than serial WRF, explicit SIMD, loop fusion, and duplicated specialization are
not justified without an end-to-end profile.

## Reproduce

```sh
./scripts/benchmark-dry-tendency-boundary-stage-fortran.sh
cargo bench -p wrf-dynamics --bench dry_tendency_boundary_stage -- --noplot
```
