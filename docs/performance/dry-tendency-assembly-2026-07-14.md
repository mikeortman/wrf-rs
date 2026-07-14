# Dry tendency assembly performance baseline

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64. The benchmark measures
WRF v4.7.1 `rk_addtend_dry`; fixture and region construction are excluded.

## Matched workload

Both implementations use a 256 × 256 × 40 physical mass grid with one stored
upper stagger, single precision, constant reused fields, and the first-substep
path. Each call updates five persistent volume tendencies, five RK volume
tendencies, and RK column mass: 26,542,080 mutable values when the extra W and
geopotential level is counted.

The script extracts the exact pinned routine from `dyn_em/module_em.F`. GNU
Fortran 16.1.0 uses `-O3 -flto`. Rust uses optimization level 3, ThinLTO, and
one codegen unit. Neither enables fast-math or an explicit native-CPU flag.

## Results

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 8.425600 ms median | 1.00× |
| Rust, one worker | 18.625 ms | 0.45× |
| Rust, four workers | 4.9522 ms | 1.70× faster |
| Rust, 16 workers | 2.5235 ms | 3.34× faster |

Fortran's eleven samples ranged from 8.281500 to 8.913450 ms. The portable
Rust Criterion intervals were 18.457–18.845 ms, 4.9230–4.9790 ms, and
2.4822–2.5808 ms for one, four, and 16 workers. A preceding FatLTO screen was
statistically indistinguishable, so the repository keeps its portable ThinLTO
default.

The serial gap is largely memory traffic and safe scheduling across six passes.
The paired-output scheduler already updates each RK/persistent pair together,
without scratch or a second first-substep pass. Because ordinary 4- and
16-worker execution beats serial WRF, further fusion or explicit SIMD waits
for a coupled trajectory profile.

## Allocation and memory behavior

After warm-up, 100 calls on a 64 × 64 × 40 grid record:

| Workers | First measured phase | Settled phase | Reallocations |
|---:|---:|---:|---:|
| 1 | 10 allocations, 15,200 bytes | 9 allocations, 13,680 bytes | 0 |
| 4 | 10 allocations, 15,200 bytes | 9 allocations, 13,680 bytes | 0 |
| 16 | 10 allocations, 15,200 bytes | 9 allocations, 13,680 bytes | 0 |

These are small Rayon scheduler allocations. The kernel allocates no numerical
scratch, clones no field, and mutates caller-owned contiguous storage directly.

## Reproduce

```sh
./scripts/benchmark-dry-tendency-assembly-fortran.sh
cargo bench -p wrf-dynamics --bench dry_tendency_assembly -- --noplot
cargo run -p wrf-dynamics --release --example measure_dry_tendency_assembly_allocations
```
