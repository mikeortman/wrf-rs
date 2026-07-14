# Flow-dependent inflow-policy performance baseline

This compares the exact WRF v4.7.1 `flow_dep_bdy_qnn` and
`flow_dep_bdy_fixed_inflow` bodies with one safe Rust traversal parameterized
by an explicit inflow policy. The matched workload contains 200,800 classified
boundary points on a 256 × 256 × 40 domain with a five-point specified zone.

GNU Fortran 16.1.0 uses `-O3 -flto -ffp-contract=off`. Rust uses optimization
level 3, ThinLTO, and one codegen unit. Neither enables fast-math or a native
CPU target.

| Policy and implementation | Time per call | Relative to matching Fortran |
|---|---:|---:|
| Constant, WRF Fortran | 0.187580 ms median | 1.00× |
| Constant, Rust 1 worker | 0.21569 ms | 15.0% slower |
| Constant, Rust 4 workers | 0.17217 ms | 1.09× faster |
| Constant, Rust 16 workers | 0.31029 ms | 65.4% slower |
| Preserve, WRF Fortran | 0.177190 ms median | 1.00× |
| Preserve, Rust 1 worker | 0.21686 ms | 22.4% slower |
| Preserve, Rust 4 workers | 0.17267 ms | 2.6% faster |
| Preserve, Rust 16 workers | 0.30735 ms | 73.5% slower |

Constant Fortran samples ranged from 0.186440 to 0.256330 ms; preserve samples
ranged from 0.170250 to 0.186570 ms. Rust Criterion intervals were
0.21443–0.21714, 0.17122–0.17336, and 0.30763–0.31290 ms for constant policy,
and 0.21578–0.21814, 0.17161–0.17385, and 0.30495–0.30976 ms for preserve.

Both Rust policies record three scheduler allocations and 4,560 bytes across
100 warmed calls, with no reallocations, numerical workspace, or field clones.
The explicit runtime policy replaces three duplicated WRF loop families. Since
four-worker Rust is competitive or faster, policy specialization, custom
scheduling, and explicit SIMD are deferred unless integrated profiling shows a
material scalar-boundary bottleneck.
