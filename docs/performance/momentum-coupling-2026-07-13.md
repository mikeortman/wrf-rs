# Momentum-coupling CPU baseline — 2026-07-13

This matched release-mode baseline compares the Rust port of WRF
`couple_momentum` with the exact routine extracted from pinned WRF v4.7.1.

## Environment and optimization

- Apple M3 Max, 16 CPU cores, arm64 macOS
- rustc/cargo 1.96.0, LLVM 22.1.2
- GNU Fortran 16.1.0
- Rust bench profile: optimization level 3, ThinLTO, one codegen unit
- Fortran: `-O3 -flto`
- no fast-math and no explicit native-CPU flag on either side

These are comparable highest-normal optimization tiers, not identical compiler
behavior.

## Matched workload

The physical mass grid is 256 × 256 with 40 half levels. Allocated storage is
258 × 258 × 42 so all three upper C-grid staggers are present. Each call writes:

```text
257 × 256 × 40 west-east values
+ 256 × 257 × 40 south-north values
+ 256 × 256 × 41 vertical values
= 7,950,336 outputs
```

Both implementations use the same single-precision initialization, mass and
map-factor fields, vertical coefficients, domain/tile bounds, expression order,
and reusable output storage.

Commands:

```sh
cargo bench -p wrf-dynamics --bench momentum_coupling -- --noplot
./scripts/benchmark-momentum-coupling-fortran.sh
```

## Accepted Rust results

Criterion central estimates and 95% confidence intervals:

| Workers | Time | Output throughput | Speedup vs. 1 worker |
|---:|---:|---:|---:|
| 1 | 1.3679 ms `[1.3523, 1.3840]` | 5.8119 Goutput/s | 1.00× |
| 4 | 654.95 µs `[643.76, 667.42]` | 12.139 Goutput/s | 2.09× |
| 16 | 1.4425 ms `[1.4256, 1.4604]` | 5.5116 Goutput/s | 0.95× |

Four workers are best. Sixteen workers add scheduler and heterogeneous-core
pressure to a streaming kernel that already approaches memory limits.

## Matched optimized Fortran

The Fortran harness performs 20 excluded warm-up calls, then eleven samples of
40 calls:

```text
1.062425  1.025500  1.059025  1.152625  1.121075  1.130975
1.196125  1.235425  1.229050  1.183550  1.276675
```

The median is 1.152625 ms and the observed range is
`[1.025500, 1.276675]` ms. One-worker Rust is 18.7% slower than serial Fortran.
Four-worker Rust is 1.76× faster than the upstream routine, which contains no
OpenMP directives.

## Safe hot-loop correction

The first parity-correct Rust implementation indexed global field slices for
every output. It measured 6.1893 ms with one worker, 2.8361 ms with four, and
3.0531 ms with 16. Those results were 5.4× slower than serial Fortran and did
not meet the project performance gate.

The accepted implementation validates once, forms equal-length active row
slices, and traverses them with safe iterators. This removes repeated global
bounds checks and gives LLVM a vector-friendly loop without changing the WRF
multiply/add/divide order. The direct 3,780-value oracle stayed bit exact. The
rewrite improved one- and four-worker timings by about 77%.

The remaining serial difference is close enough given the standard four-worker
path, bounded allocation behavior, and lack of an end-to-end hotspot profile.
No explicit SIMD or target-specific implementation is being added.

## Allocation evidence

The release allocation harness performs 100 warm-up calls and two measured
100-call phases. Every phase at 1, 4, and 16 workers records five allocations
totaling 7,600 bytes and zero reallocations. That is 0.05 allocation and 76
bytes per call, independent of field size and worker count. The numerical
kernel allocates no field-sized, row-sized, or vertical scratch.

## Scope

This is an isolated ARW utility routine on one machine. It does not establish
whole-model performance or forecast throughput.
