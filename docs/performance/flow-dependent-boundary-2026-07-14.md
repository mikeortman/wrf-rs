# Flow-dependent specified-boundary performance baseline

This compares the exact WRF v4.7.1 `flow_dep_bdy` body with safe Rust on a
256 × 256 horizontal mass domain, 40 active half levels, and a five-point
specified zone. Each call classifies and writes 200,800 points from coupled
U/V velocity signs.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, and one codegen unit. Neither enables fast-math or a native
CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 0.189720 ms median | 1.00× |
| Rust, 1 worker | 0.18688 ms | 1.5% faster |
| Rust, 4 workers | 0.15356 ms | 1.24× faster |
| Rust, 16 workers | 0.32330 ms | 70.4% slower |

Fortran's eleven samples ranged from 0.188190 to 0.305730 ms. Criterion
intervals were 0.18611–0.18778, 0.15218–0.15509, and 0.32024–0.32633 ms.

The kernel owns no numerical workspace and clones no field. South and north
copies borrow immutable interior scalar planes disjoint from parallel
destination planes while reading V directly. West and east copies use direct
safe indexing because WRF may clamp the scalar source row away from the
destination row.

Across 100 warmed calls, the scheduler records three allocations and 4,560
bytes, with no reallocations. Serial and four-worker Rust clear the performance
gate. The host-default pool is overhead-bound on this thin perimeter, but a
special worker policy would complicate the shared backend without integrated
evidence. Explicit SIMD and custom scheduling are therefore deferred.
