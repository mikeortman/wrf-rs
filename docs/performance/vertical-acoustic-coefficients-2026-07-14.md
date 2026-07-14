# Vertical acoustic coefficient performance baseline

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64. The benchmark measures
WRF v4.7.1 `calc_coef_w`; fixture construction is excluded.

## Matched workload and toolchains

Both implementations process 256 × 256 independent columns with 40 mass half
levels and the additional top full level. All inputs and three output fields
are allocated once and reused. The nonrigid top boundary is representative;
the rigid-lid branch changes one multiplication per column.

The script extracts the exact pinned routine. GNU Fortran 16.1.0 uses `-O3
-flto`. Rust uses optimization level 3, ThinLTO, and one codegen unit. Neither
side enables fast-math or an explicit native-CPU flag.

## Results

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 1.867500 ms median | 1.00× |
| Rust, one worker | 14.608 ms | 7.82× slower |
| Rust, four workers | 3.8912 ms | 2.08× slower |
| Rust, 16 workers | 1.7109 ms | 1.09× faster |

The eleven Fortran samples span 1.829250–1.956000 ms. Rust Criterion intervals
are 14.536–14.686 ms, 3.8822–3.9007 ms, and 1.6899–1.7329 ms.

The standard host-parallel path clears the matched-Fortran gate. The serial
gap indicates stronger Fortran vectorization across contiguous west–east
levels, but explicit SIMD is deferred until a coupled acoustic trajectory
shows this routine is a material limiter.

## Traversal correction

The first parity-correct Rust implementation completed one strided vertical
column before moving west–east and measured 27.881 ms serially. Reordering each
owned south–north plane to the WRF loop direction—level outer, contiguous
west–east inner—preserved all 3,024 oracle values and improved serial time by
47.6% to 14.608 ms. No arithmetic, ownership, or boundary behavior changed.

## Allocation behavior

After warm-up, every 100 calls on 64 × 64 columns with 40 mass levels records
three allocations totaling 4,560 bytes at one, four, and 16 workers. There are
no reallocations, field clones, per-column buffers, or numerical scratch
allocations. The two scheduler phases operate directly on caller-owned output
storage.

## Reproduce

```sh
./scripts/benchmark-vertical-acoustic-coefficients-fortran.sh
cargo bench -p wrf-dynamics --bench vertical_acoustic_coefficients -- --noplot
cargo run -p wrf-dynamics --release --example measure_vertical_acoustic_coefficient_allocations
```
