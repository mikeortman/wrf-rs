# Acoustic horizontal-momentum performance baseline

This compares the exact WRF v4.7.1 `advance_uv` body with safe Rust on the same
256 × 256 × 40 nonhydrostatic grid, including upper U/V stagger points.

GNU Fortran 16.1.0 uses `-O3 -flto`. Rust uses optimization level 3, ThinLTO,
and one codegen unit. Neither enables fast-math or a native CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 7.569 ms median | 1.00× |
| Rust, 1 worker | 71.852 ms | 9.49× slower |
| Rust, 4 workers | 18.850 ms | 2.49× slower |
| Rust, 16 workers | 7.802 ms | 1.03× slower |

Fortran samples ranged from 7.039 to 8.741 ms. Criterion intervals were
71.539–72.192, 18.818–18.885, and 7.538–8.141 ms.

Rust allocates no tile-sized numerical scratch: full-level pressure,
pressure-gradient, and damping terms are computed and consumed locally. The
default path is operationally close to optimized Fortran, so SIMD and more
complex fusion are deferred until coupled profiling shows a model-level gain.
