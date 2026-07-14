# Acoustic pressure diagnosis performance baseline

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64. The benchmark measures
WRF v4.7.1 `calc_p_rho`; fixture construction is excluded.

## Matched workload and toolchains

Both implementations process a 256 × 256 × 40 mass grid with one allocated
upper geopotential level. The same reused single-precision fields execute the
pressure-history initialization path. Nonhydrostatic and hydrostatic modes are
timed separately.

The script extracts the exact pinned routine. GNU Fortran 16.1.0 uses `-O3
-flto`. Rust uses optimization level 3, ThinLTO, and one codegen unit. Neither
enables fast-math or an explicit native-CPU flag.

## Results

| Mode | Implementation | Time per call | Relative to serial Fortran |
|---|---|---:|---:|
| nonhydrostatic | WRF Fortran, serial | 1.529500 ms median | 1.00× |
| nonhydrostatic | Rust, one worker | 1.8319 ms | 19.8% slower |
| nonhydrostatic | Rust, four workers | 0.81126 ms | 1.89× faster |
| nonhydrostatic | Rust, 16 workers | 1.5168 ms | 0.8% faster |
| hydrostatic | WRF Fortran, serial | 1.602750 ms median | 1.00× |
| hydrostatic | Rust, one worker | 2.0816 ms | 29.9% slower |
| hydrostatic | Rust, four workers | 0.95950 ms | 1.67× faster |
| hydrostatic | Rust, 16 workers | 2.1141 ms | 31.9% slower |

Fortran sample ranges were 1.512550–2.006750 ms nonhydrostatic and
1.563400–1.765500 ms hydrostatic. Rust Criterion intervals were
1.8075–1.8596 ms, 0.80409–0.82094 ms, and 1.3982–1.6060 ms for the three
nonhydrostatic worker counts. Hydrostatic intervals were 2.0569–2.1137 ms,
0.94956–0.97188 ms, and a noisy 1.6291–2.8578 ms.

Four workers are the best measured configuration in both modes. The standard
multithreaded implementation already beats optimized serial Fortran, while
oversubscribing this bandwidth-sensitive kernel adds dispatch variability. No
per-kernel worker policy, explicit SIMD, unsafe fusion, or target-specific flag
is justified without a coupled acoustic trajectory profile.

## Hydrostatic traversal correction

The first parity-correct hydrostatic recurrence visited all levels for one X
column before moving to the next column. That traversal was safe but strided in
XZY storage and measured 8.63 ms serially. Reordering the owned plane to WRF's
level-major, contiguous-X traversal preserved every exact oracle value and
improved serial time to 2.08 ms. This was a layout correction, not changed
arithmetic.

## Allocation behavior

After warm-up, 100 calls on a 64 × 64 × 40 grid record:

| Mode | Phase | Allocations | Bytes | Reallocations |
|---|---|---:|---:|---:|
| nonhydrostatic | first/settled maximum | 5 | 4,672 | 0 |
| hydrostatic | first | 5 | 7,600 | 0 |
| hydrostatic | settled | 4 | 6,080 | 0 |

Counts are independent of field size and worker count apart from one observed
small Rayon bookkeeping variation. The kernel allocates no numerical scratch,
clones no field, and mutates caller-owned contiguous storage.

## Reproduce

```sh
./scripts/benchmark-acoustic-pressure-fortran.sh
cargo bench -p wrf-dynamics --bench acoustic_pressure -- --noplot
cargo run -p wrf-dynamics --release --example measure_acoustic_pressure_allocations
```
