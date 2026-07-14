# Acoustic flux accumulation performance baseline

This compares the exact WRF v4.7.1 `sumflux` body with safe Rust for a complete
three-substep sequence on a 256 × 256 × 40 mass grid. Fixture construction is
excluded. Each implementation clears first-substep storage, accumulates three
current flux fields per substep, and finalizes all staggered averages.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, one codegen unit, and portable code generation. Neither side
enables fast-math or a native CPU target.

| Implementation | Time per three-substep sequence | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 5.513750 ms median | 1.00× |
| Rust, 1 worker | 26.048 ms | 4.72× slower |
| Rust, 4 workers | 7.2084 ms | 30.7% slower |
| Rust, 16 workers | 3.6192 ms | 1.52× faster |

Fortran's eleven samples were 6.643500, 6.285500, 5.795000, 5.720250,
5.701750, 5.447250, 5.401000, 5.383500, 5.208000, 5.163000, and 5.513750 ms.
Criterion intervals were 25.922–26.187, 7.1893–7.2301, and 3.5976–3.6424 ms.

The kernel uses no numerical scratch and clones no fields. Every settled 100-
sequence allocation measurement recorded 19 scheduler allocations totaling
28,880 bytes, no reallocations, for one, four, and sixteen workers.

The ordinary host-parallel path is materially faster than optimized serial
Fortran. Per project policy, explicit SIMD, pass fusion, and specialized worker
selection stop here until the coupled acoustic trajectory identifies a model-
level bottleneck.
