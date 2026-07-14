# Complete local acoustic-trajectory performance estimate

Date: 2026-07-14. Host: Apple M3 Max. Toolchains: Rust 1.96.0 and GNU Fortran
16.1.0.

## Workload and optimization equivalence

The estimate composes the already measured 256 × 256 × 40 stage benchmarks in
the exact local trajectory count:

- one `small_step_prep`;
- four nonhydrostatic `calc_p_rho` calls;
- one `calc_coef_w`;
- three calls each to `advance_uv`, `advance_mu_t`, and `advance_w`; and
- one measured three-call `sumflux` sequence.

Fortran stages use `-O3 -flto` and disable contraction where required for
parity. Rust stages use optimization level 3, ThinLTO, and one codegen unit.
Neither side enables fast-math or a native-CPU target. This is the same matched
optimization policy used for every source stage.

## Aggregate result

| Implementation | Sum of matched stage medians | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 108.423 ms | 1.00× |
| Rust, one worker | 563.428 ms | 5.20× slower |
| Rust, four workers | 150.770 ms | 39.1% slower |
| Rust, 16 workers | 73.949 ms | 1.47× faster |

These numbers are an arithmetic composition of independently measured stage
medians, not a new wall-clock measurement of one fused driver. They exclude
halo exchange, physical boundaries, polar filtering, nested forcing, and
Registry state binding on both sides. They also exclude the Rust trajectory's
one structural preflight, so the estimate must not be presented as an
end-to-end model speedup.

## Decision

The ordinary 16-worker Rust path is already materially faster in the matched
local computation estimate. No explicit SIMD, unsafe fusion, or specialized
worker-count policy is justified for the composition layer. A direct integrated
benchmark belongs with the next driver slice, where communication and boundary
insertion costs can be measured rather than hidden.
