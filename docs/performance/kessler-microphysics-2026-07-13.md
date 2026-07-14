# Kessler microphysics performance baseline

## Environment and workload

- Date: 2026-07-13
- Machine: Apple M3 Max, macOS 26.2 arm64
- Rust: rustc 1.96.0, release `opt-level=3`, ThinLTO, one codegen unit
- Fortran: GNU Fortran 14.2.0, `-O3 -flto`
- Fast math: disabled in both implementations
- Domain: 128 × 128 horizontal columns × 40 vertical levels
- Work: 655,360 grid points per call
- Physics time step: 60 seconds

Both implementations receive the same mixed cloud/rain field. Mutable fields
are restored before each timed call; restoration is outside the timer. The
pinned `module_mp_kessler.F` is compiled directly.

## Timing

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 31.7804 ms median | 1.00× |
| Rust, 1 worker | 30.944 ms estimate `[30.601, 31.340]` | 1.03× faster |
| Rust, 4 workers | 8.9176 ms `[8.8592, 8.9814]` | 3.56× faster |
| Rust, 16 workers | 5.0144 ms `[4.8936, 5.1846]` | 6.34× faster |

The serial implementations are operationally equal in performance. Per project
policy, this is a stopping signal: no SIMD or more complex memory layout is
justified without an end-to-end profile showing Kessler as a material hotspot.

## Memory behavior

The reusable numerical workspace contains one production value per grid point
and one 40-level terminal-velocity buffer per worker:

| Workers | Numerical scratch | Allocations per 100 settled calls | Reallocations |
|---:|---:|---:|---:|
| 1 | 2,621,600 bytes | 3 allocations / 4,560 bytes | 0 |
| 4 | 2,622,080 bytes | 3 allocations / 4,560 bytes | 0 |
| 16 | 2,624,000 bytes | 3 allocations / 4,560 bytes | 0 |

The three small allocations are persistent Rayon/crossbeam scheduling activity,
not numerical scratch. The production field and per-worker vertical buffers
are allocated during workspace creation and reused.

## Interpretation

The Rust implementation is not a line-by-line port. It retains the same
single-precision operation order within each column while replacing Fortran's
call-local automatic arrays with an explicit reusable workspace. Parallelism is
over independent south-north rows; sedimentation remains vertically ordered.

These results describe one isolated scheme call. They do not include the WRF
physics driver, state packing, halos, I/O, or model-step synchronization and do
not imply a whole-model speedup.

## Reproduce

```sh
./scripts/run-kessler-oracle.sh
./scripts/benchmark-kessler-fortran.sh
cargo bench -p wrf-physics --bench kessler_microphysics -- --noplot
cargo run --release -p wrf-physics --example measure_kessler_allocations
```
