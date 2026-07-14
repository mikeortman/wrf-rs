# Dry boundary-relaxation performance baseline

This compares WRF v4.7.1 `relax_bdy_dry` plus its exact `mass_weight` and
`relax_bdytend` dependencies with the safe Rust orchestration capability. The
matched nested workload uses a 256 × 256 × 40 mass grid and performs 1,209,216
five-point relaxation updates plus 7,995,392 mass-weighted points per call.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust 1.96.0 uses
optimization level 3, ThinLTO, and one codegen unit. Neither implementation
enables fast-math or a native CPU target. Measurements ran on an Apple M3 Max
under macOS 26.2 arm64.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran | 4.1218 ms median | 1.00× |
| Rust, 1 worker | 20.101 ms | 4.88× slower |
| Rust, 4 workers | 5.5859 ms | 35.5% slower |
| Rust, 16 workers | 4.1620 ms | 1.0% slower |

The stable Fortran samples ranged from 3.9272 to 4.4852 ms. Rust's Criterion
confidence intervals were 19.950–20.294 ms, 5.5627–5.6089 ms, and
4.1349–4.1903 ms for one, four, and sixteen workers. A later Fortran repetition
showed system/thermal drift and is not substituted for the first matched run.

Across 100 warmed calls, the default Rust path records 14 allocations and 14
deallocations totaling 21,280 bytes, with no reallocations. Those allocations
belong to persistent-pool scheduling; the operation reuses its caller-owned
tile-halo workspace and performs no field clones or per-call field-sized
allocation.

The Rust capability also skips mass weighting for inactive tiles and empty
relaxation bands. WRF still fills its automatic tile-sized scratch array on
those paths even though no tendency reads the result. This changes no
observable output. Default multithreaded Rust is within 1% of optimized serial
Fortran, so explicit SIMD and further specialization stop under the project's
close-enough rule.

Reproduce with:

```sh
./scripts/benchmark-dry-boundary-relaxation-fortran.sh
cargo bench -p wrf-dynamics --bench dry_boundary_relaxation -- --noplot
```
