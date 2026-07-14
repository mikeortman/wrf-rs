# Specified-boundary geopotential performance baseline

This compares the exact WRF v4.7.1 `spec_bdyupdate_ph` body with safe Rust on a
256 × 256 horizontal mass domain, 41 active full levels, and a five-point
specified zone. Each call updates 205,820 points.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, and one codegen unit. Neither enables fast-math or a native
CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 0.310400 ms median | 1.00× |
| Rust, 1 worker | 0.15437 ms | 2.01× faster |
| Rust, 4 workers | 0.055178 ms | 5.63× faster |
| Rust, 16 workers | 0.095639 ms | 3.25× faster |

Fortran's eleven samples ranged from 0.301490 to 0.337520 ms. Criterion
intervals were 0.15322–0.15570, 0.054655–0.055777, and
0.094880–0.096422 ms.

WRF declares `mu_old` over the complete tile although each element is assigned
and consumed immediately inside a boundary loop. Rust keeps that value scalar
and processes only the direct boundary ranges. This is a combined compiler and
implementation comparison; it does not isolate the automatic array's cost.

Across 100 warmed calls, the scheduler records two allocations and 3,040
bytes, with no reallocations. The kernel allocates no numerical scratch and
clones no fields. Ordinary safe Rust already exceeds optimized serial Fortran,
so explicit SIMD and per-call worker tuning are deferred.
