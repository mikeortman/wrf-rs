# Specified-boundary tendency-update performance baseline

This compares the exact WRF v4.7.1 `spec_bdyupdate` body with safe Rust on a
256 × 256 × 40 mass grid and a five-point specified zone. Each call updates
200,800 points in the four trapezoid/side boundary regions.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, and one codegen unit. Neither build enables fast-math or a
native CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 0.144940 ms median | 1.00× |
| Rust, 1 worker | 0.096968 ms | 1.49× faster |
| Rust, 4 workers | 0.037367 ms | 3.88× faster |
| Rust, 16 workers | 0.089821 ms | 1.61× faster |

Fortran's eleven samples ranged from 0.133350 to 0.162710 ms. Criterion
intervals were 0.096231–0.097789, 0.036874–0.037964, and
0.089306–0.090331 ms.

The first exact Rust version scanned every horizontal point in every active
plane and measured 8.2470 ms with one worker. Replacing that scan with four
direct, source-ordered boundary ranges reduced work to WRF's boundary-only
complexity without changing an oracle bit.

Across 100 warmed calls, the scheduler records at most four allocations and
3,152 bytes, with no reallocations. The kernel allocates no numerical scratch
and clones no fields. The standard implementation already exceeds optimized
serial Fortran, so explicit SIMD and per-call worker tuning are deferred.
