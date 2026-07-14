# Physical boundary-zone performance

Date: 2026-07-14. Machine: Apple M3 Max with 12 performance and 4 efficiency
cores, 128 GB unified memory, macOS 26.2 arm64. Toolchains: Rust 1.96.0 with
LLVM 22.1.2 and GNU Fortran 16.1.0. No affinity or NUMA policy was applied.

## Matched workload

Both programs apply the specified mass-field form of `set_physical_bc3d` to a
256 × 256 × 40 physical grid in 265 × 265 × 42 storage. The full tile contacts
all sides and writes 126,960 boundary values per call. Allocation,
initialization, and deallocation are outside timing; the repeated operation is
idempotent for the uniform input.

Fortran uses `-O3 -flto -ffp-contract=off`. Rust uses the workspace bench
profile: optimization level 3, ThinLTO, and one codegen unit. Neither side uses
fast-math or a native-CPU target. Fortran records 31 samples of 100 calls each;
Criterion records warmed samples for backends configured with 1, 4, and 16
workers.

## Correctness

The separate direct oracle covers periodic, specified, nested, partial,
inactive, staggered, and exceptional-value cases. All finite results,
infinities, and signed zeros have zero bit, absolute, and relative error; NaNs
match by IEEE class. Rust preserves WRF's west-east then south-north operation
order.

## Results

| Implementation | Median or Criterion estimate | Relative to serial Fortran |
|---|---:|---:|
| WRF Fortran, serial | 24.600 µs median | 1.00× |
| Rust, one-worker backend | 43.040 µs | 1.75× slower |
| Rust, four-worker backend | 43.302 µs | 1.76× slower |
| Rust, 16-worker backend | 43.305 µs | 1.76× slower |

Fortran's 31 samples span 22.890–26.280 µs. Criterion intervals are
42.846–43.288 µs, 43.030–43.619 µs, and 43.027–43.639 µs. The Rust throughput
is about 2.93 billion assigned values per second.

The worker counts intentionally converge: profiling by measurement showed that
dispatching this thin perimeter traversal costs more than it saves. The final
implementation keeps the exact source-order loop local. Across 100 warmed calls
at every worker count it performs zero allocations and zero reallocations.

The remaining serial gap includes Rust's typed shape validation and geometry
construction on every public call, while the Fortran driver invokes the raw
routine. No cache-miss or generated-instruction counters were collected, so
specialized geometry caching or loop duplication is not justified yet. The
complete acoustic-stage profile is the appropriate place to decide whether
this small boundary cost is material.

## Reproduce

```sh
./scripts/benchmark-physical-boundary-fortran.sh
cargo bench -p wrf-dynamics --bench physical_boundary -- --noplot
cargo run -p wrf-dynamics --release --example measure_physical_boundary_allocations
```
