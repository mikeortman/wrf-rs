# Implicit acoustic vertical-momentum performance baseline

This compares the exact WRF v4.7.1 `advance_w` body with safe Rust on the same
256 × 256 × 40 mass grid. Both sides include the lower terrain boundary,
geopotential transport, forward/back tridiagonal sweeps, upper damping, and the
full upper vertical-momentum level.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, one codegen unit, and portable code generation. Neither side
enables fast-math or a native CPU target.

| Implementation | Time per call | Relative to Fortran |
|---|---:|---:|
| WRF Fortran, serial | 16.745 ms median | 1.00× |
| Rust, 1 worker | 61.295 ms | 3.66× slower |
| Rust, 4 workers | 16.084 ms | 1.04× faster |
| Rust, 16 workers | 6.621 ms | 2.53× faster |

Fortran's eleven samples were 16.287, 16.745, 16.546, 26.746, 19.592,
16.737, 17.515, 16.121, 17.272, 17.248, and 16.014 ms. Criterion intervals
were 61.074–61.480, 16.050–16.119, and 6.501–6.703 ms.

The caller-owned RHS workspace contains 2,795,688 `f32` values, or 10.67 MiB,
for the guarded benchmark shape. It is allocated once during setup. Every 100
settled 16-worker calls recorded four scheduler allocations totaling 6,080
bytes, four matching deallocations, and no reallocations. The numerical kernel
allocates no field storage and clones no fields.

Four-worker Rust is already operationally equal to optimized serial Fortran,
and the standard host-parallel path is materially faster. Per project policy,
explicit SIMD, column-layout transposition, and additional loop fusion stop
here until a coupled acoustic trajectory identifies this routine as a model-
level bottleneck.
