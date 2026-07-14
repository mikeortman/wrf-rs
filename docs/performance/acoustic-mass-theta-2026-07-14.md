# Acoustic mass, omega, and theta performance baseline

This compares the exact WRF v4.7.1 `advance_mu_t` body with safe Rust on the
same 256 × 256 × 40 mass grid, including upper U/V and full-level storage.

GNU Fortran 16.1.0 uses `-O3 -flto`. Rust uses optimization level 3, ThinLTO,
and one codegen unit. Neither enables fast-math or a native CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 5.368 ms median | 1.00× |
| Rust, 1 worker | 29.960 ms | 5.58× slower |
| Rust, 4 workers | 8.070 ms | 1.50× slower |
| Rust, 16 workers | 4.241 ms | 1.27× faster |

Fortran's eleven samples ranged from 5.216 to 6.601 ms. Criterion intervals
were 29.795–30.135, 8.043–8.097, and 4.192–4.299 ms.

Rust allocates no numerical scratch. It temporarily stores horizontal flux
divergence in the required previous-temperature output and prior column mass in
the required coupled-mass output, consuming each before writing its final
specified value. Fields are never cloned. The default parallel path already
beats the matched optimized Fortran routine, so explicit SIMD and a more
complex multi-output scheduler are deferred.
