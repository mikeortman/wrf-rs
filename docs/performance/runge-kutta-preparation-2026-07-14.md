# Runge-Kutta preparation performance baseline

Date: 2026-07-14. Machine: Apple M3 Max, macOS arm64. The benchmark measures
the complete seven-diagnostic preparation pass used before an ARW Runge-Kutta
step; fixture construction and validation-region construction are excluded.

## Matched workload

Both implementations use a 256 × 256 × 40 physical mass grid, one stored halo
or upper-stagger point at each side, two active moisture species, single
precision, nonperiodic boundaries, and the same constant input fields. Each
call runs, in WRF order:

1. communicated-tile full column mass;
2. west-east and south-north staggered column mass;
3. three-component momentum coupling;
4. dry-air omega diagnosis;
5. three moisture coefficients;
6. full inverse density; and
7. pressure-point geopotential.

The exact pinned WRF v4.7.1 routine bodies are compiled with GNU Fortran 16.1.0
using `-O3 -flto`. Rust uses the workspace bench profile: optimization level 3,
ThinLTO, and one codegen unit. Neither build enables fast-math or an explicit
native-CPU flag.

## Results

| Implementation | Time per call | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 6.067100 ms median | 1.00× |
| Rust, one worker | 10.092 ms | 0.60× |
| Rust, four workers | 3.3025 ms | 1.84× faster |
| Rust, 16 workers | 4.5476 ms | 1.33× faster |

Fortran's eleven raw samples ranged from 5.997000 to 6.636100 ms. Criterion's
95% intervals were 10.023–10.162 ms, 3.2904–3.3133 ms, and 4.4933–4.5953 ms
for one, four, and 16 workers respectively.

The integrated serial gap is mainly the accumulated cost of routines already
documented individually, especially omega and moisture coefficients. Four
workers are best on this machine; 16 workers add scheduling and bandwidth
pressure. Because the standard multithreaded path beats serial WRF and the port
does not yet have a coupled trajectory profile, cross-stage fusion, custom
scheduling, and additional explicit SIMD are deliberately deferred.

## Allocation and memory behavior

After 100 warm-up calls, each 100-call measured phase records:

| Workers | Allocations | Reallocations | Bytes |
|---:|---:|---:|---:|
| 1 | 19 | 0 | 28,880 |
| 4 | 19 | 0 | 28,880 |
| 16 | 19 | 0 | 28,880 |

These are persistent-pool scheduler allocations of 1,520 bytes each. The
pipeline allocates no numerical scratch, clones no field, and creates no
temporary full-domain output. Its all-stage preflight consists only of borrowed
views and typed metadata.

## Reproduce

```sh
./scripts/benchmark-runge-kutta-preparation-fortran.sh
cargo bench -p wrf-dynamics --bench runge_kutta_preparation -- --noplot
cargo run -p wrf-dynamics --release --example measure_runge_kutta_preparation_allocations
```
