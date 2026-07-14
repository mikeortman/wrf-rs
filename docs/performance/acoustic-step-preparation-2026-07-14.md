# Acoustic small-step preparation performance baseline

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64. The benchmark measures
WRF v4.7.1 `small_step_prep`; fixture and region construction are excluded.

## Matched workload

Both implementations execute the first-substep path on a 256 × 256 × 40 mass
grid with upper U/V horizontal staggers and one upper W/geopotential level.
Each call writes 45,543,936 values across previous, current, saved, diagnostic,
and column-mass outputs. Inputs and output storage are reused between calls.

The script extracts the exact routine from `dyn_em/module_small_step_em.F`.
GNU Fortran 16.1.0 uses `-O3 -flto`. Rust uses optimization level 3, ThinLTO,
and one codegen unit. Neither build enables fast-math or an explicit native-CPU
flag.

## Results

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 5.877800 ms median | 1.00× |
| Rust, one worker | 26.123 ms | 4.44× slower |
| Rust, four workers | 7.4138 ms | 26.1% slower |
| Rust, 16 workers | 6.5595 ms | 11.6% slower |

Fortran's eleven samples ranged from 5.731300 to 7.147600 ms. Rust Criterion
intervals were 25.869–26.388 ms, 7.3805–7.4481 ms, and 6.3699–6.7130 ms for
one, four, and 16 workers.

This kernel makes many source-ordered passes over large fields. Safe Rust's
serial scheduling and bounds contracts are visible in the one-worker result,
while ordinary parallel execution brings the implementation into operational
proximity to optimized Fortran. The project rule is to stop tuning at this
point unless a coupled trajectory profile identifies the routine as a material
bottleneck. No explicit SIMD, unsafe fusion, target-specific flags, or more
complex ownership model is justified by this isolated result.

## Allocation and memory behavior

After warm-up, 100 calls on a 64 × 64 × 40 grid record:

| Workers | First measured phase | Settled phase | Reallocations |
|---:|---:|---:|---:|
| 1 | 29 allocations, 44,080 bytes | 28 allocations, 42,560 bytes | 0 |
| 4 | 29 allocations, 44,080 bytes | 28 allocations, 42,560 bytes | 0 |
| 16 | 29 allocations, 44,080 bytes | 28 allocations, 42,560 bytes | 0 |

These are bounded Rayon scheduler allocations across the routine's independent
passes. The kernel allocates no numerical scratch, clones no field, and mutates
caller-owned contiguous storage directly.

## Reproduce

```sh
./scripts/benchmark-acoustic-step-preparation-fortran.sh
cargo bench -p wrf-dynamics --bench acoustic_step_preparation -- --noplot
cargo run -p wrf-dynamics --release --example measure_acoustic_step_preparation_allocations
```
