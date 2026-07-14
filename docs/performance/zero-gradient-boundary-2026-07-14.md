# Zero-gradient specified-boundary performance baseline

This compares the exact WRF v4.7.1 `zero_grad_bdy` body with safe Rust on a
256 × 256 horizontal mass domain, 41 active full levels, and a five-point
specified zone. Each call copies 205,820 points from the nearest independent
interior row or column.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, and one codegen unit. Neither enables fast-math or a native
CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 0.134000 ms median | 1.00× |
| Rust, 1 worker | 0.14306 ms | 6.8% slower |
| Rust, 4 workers | 0.11030 ms | 1.21× faster |
| Rust, 16 workers | 0.25671 ms | 1.92× slower |

Fortran's eleven samples ranged from 0.131830 to 0.148520 ms. Criterion
intervals were 0.14221–0.14406, 0.10961–0.11109, and 0.25335–0.25988 ms.

The kernel owns no numerical workspace and clones no field. South and north
copies borrow immutable source planes disjoint from parallel destination
planes. West and east copies use direct safe indexing because WRF can clamp
the source row away from the destination row.

Across 100 warmed calls, the scheduler records three allocations and 4,560
bytes, with no reallocations. Serial Rust is close to optimized Fortran and the
four-worker path is faster. The host-default pool is overhead-bound on this
thin perimeter operation, but choosing a custom worker count inside one kernel
would add complexity without an end-to-end profile showing value. Explicit
SIMD and custom scheduling are therefore deferred.
