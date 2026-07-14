# Moisture-coefficient performance baseline — 2026-07-13

## Result

On the matched 256 × 256 × 40 workload with six active moisture species,
optimized serial WRF Fortran measured a 5.221150 ms median. Safe Rust measured
7.1239 ms with one worker, 2.0418 ms with four, and 3.4246 ms with 16.
Four-worker Rust is 2.56× faster than serial Fortran; the default all-core path
is 1.52× faster.

Serial Rust is 36.4% slower, but both standard parallel measurements beat the
source routine and the kernel allocates no numerical scratch. Per the project
stopping rule, no explicit SIMD or custom scheduling is being added unless an
integrated ARW profile identifies `calc_cq` as a material bottleneck.

## Matched workload

- physical mass grid: 256 × 256;
- half levels: 40;
- storage: 258 × 258 × 42, including halos and upper stagger points;
- active physical moisture species: six;
- WRF scalar padding slot: one poisoned field, excluded from accumulation;
- coefficient outputs per call: 7,819,264;
- all horizontal and vertical tiles reach their upper stagger;
- identical single-precision species initialization and accumulation order;
- outputs and inputs are allocated once and reused.

The Fortran driver calls the exact extracted WRF v4.7.1 routine. It uses GNU
Fortran 16.1.0 with `-O3 -flto`, without fast-math or a native-CPU flag. The
Rust benchmark uses the workspace bench profile (`opt-level=3`, thin LTO, one
codegen unit) and Criterion 0.7.

## Measurements

| Implementation | Time per call | Throughput | Relative to serial Fortran |
|---|---:|---:|---:|
| GNU Fortran, serial | 5.221150 ms median | 1.498 Goutput/s | 1.00× |
| Rust, 1 worker | 7.1239 ms `[7.0845, 7.1668]` | 1.0976 Goutput/s | 36.4% slower |
| Rust, 4 workers | 2.0418 ms `[2.0297, 2.0546]` | 3.8297 Goutput/s | 2.56× faster |
| Rust, 16 workers | 3.4246 ms `[3.3379, 3.5187]` | 2.2833 Goutput/s | 1.52× faster |

Raw Fortran milliseconds per call:

```text
5.216050 5.192750 5.383950 5.510350 5.375850 5.331550
5.161350 5.192800 5.213850 5.330300 5.221150
```

The median is 5.221150 ms and the observed range is
`[5.161350, 5.510350]` ms.

The 16-worker confidence interval is wide and its median is slower than four
workers. This routine launches three row-parallel component passes over a
moderate domain, so synchronization and memory-bandwidth costs dominate before
all cores are useful. The default all-core path still outperforms serial
Fortran; adding a special worker policy for one kernel would complicate the API
without model-level evidence.

## Memory and allocations

WRF declares automatic `qtot(its:ite)` scratch and clears it for every active
row. Rust clears each active output row, accumulates species into that row in
WRF order, then converts the total to its final coefficient. All temporary
values are overwritten before return, so no field-, tile-, or row-sized
numerical scratch is allocated.

For each worker count, both measured 100-call phases recorded:

| Workers | Phase | Allocations | Reallocations | Bytes |
|---:|---|---:|---:|---:|
| 1 | first | 5 | 0 | 7,600 |
| 1 | settled | 5 | 0 | 7,600 |
| 4 | first | 5 | 0 | 7,600 |
| 4 | settled | 5 | 0 | 7,600 |
| 16 | first | 5 | 0 | 7,600 |
| 16 | settled | 5 | 0 | 7,600 |

These are field-size-independent scheduler allocations: 0.05 allocation and
76 bytes per complete three-component call.

## Reproduction

```sh
./scripts/benchmark-moisture-coefficients-fortran.sh
cargo bench -p wrf-dynamics --bench moisture_coefficients -- --noplot
cargo run -p wrf-dynamics --release --example measure_moisture_coefficient_allocations
```

## Scope

This is one isolated routine on one machine. It does not establish whole-model
performance, forecast throughput, or GPU behavior.
