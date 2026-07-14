# Omega-diagnosis performance baseline — 2026-07-13

## Result

On the matched 256 × 256 × 40 complete-column workload, optimized serial WRF
Fortran measured a 1.832250 ms median. Safe Rust measured 5.0201 ms with one
worker, 1.3306 ms with four, and 666.90 µs with 16. Four-worker Rust is 1.38×
faster than serial Fortran; 16-worker Rust is 2.75× faster.

Serial Rust remains 2.74× slower. The standard backend is multithreaded, both
measured parallel configurations are faster than the source routine, and this
kernel allocates no numerical scratch. That is sufficient to stop tuning until
an integrated ARW profile shows `calc_ww_cp` is a material bottleneck.

## Matched workload

- physical mass grid: 256 × 256;
- half levels: 40, plus bottom and top full-level boundaries;
- storage: 258 × 258 × 42, including horizontal neighbors and vertical halos;
- active output values: 2,686,976;
- both horizontal tiles reach their upper boundary;
- identical single-precision velocity, mass, map-factor, coefficient, and grid
  spacing formulas;
- output and all inputs are allocated once and reused.

The Fortran driver calls the exact extracted WRF v4.7.1 routine. It uses GNU
Fortran 16.1.0 with `-O3 -flto`, without fast-math or a native-CPU flag. The
Rust benchmark uses the workspace release/bench profile (`opt-level=3`, thin
LTO, one codegen unit) and Criterion 0.7.

## Measurements

| Implementation | Time per call | Throughput | Relative to serial Fortran |
|---|---:|---:|---:|
| GNU Fortran, serial | 1.832250 ms median | 1.466 Goutput/s | 1.00× |
| Rust, 1 worker | 5.0201 ms `[4.9991, 5.0425]` | 535.24 Moutput/s | 2.74× slower |
| Rust, 4 workers | 1.3306 ms `[1.3293, 1.3320]` | 2.0193 Goutput/s | 1.38× faster |
| Rust, 16 workers | 666.90 µs `[658.13, 675.84]` | 4.0290 Goutput/s | 2.75× faster |

Raw Fortran milliseconds per call:

```text
1.788100 1.870050 1.839750 1.821550 1.783600 1.881850
1.842950 1.856000 1.832250 1.797400 1.743500
```

The median is 1.832250 ms and the observed range is
`[1.743500, 1.881850]` ms.

## Accepted layout correction

The first parity-correct Rust version owned one vertical column at a time. It
measured 17.960 ms with one worker, 4.6252 ms with four, and 2.2258 ms with 16.
That traversal repeatedly crossed strided storage and prevented useful
west-east vectorization.

The accepted implementation keeps the scientific stages separate but computes
horizontal divergence through validated, equal-length west-east row views.
The row types are grouped in `omega_diagnosis/row/`, so slice relationships are
named and checked once rather than hidden in index arithmetic. The subsequent
vertical integration remains column-local because it has a real recurrence.
Every focused Fortran value stayed identical. Representative times improved by
about 70% without explicit SIMD or unsafe code.

## Memory and allocations

WRF creates automatic tile arrays for `muu`, `muv`, `divv`, and `dmdt` on every
call. Rust computes staggered mass directly from borrowed rows and temporarily
stores divergence and column mass tendency in active output levels that are
overwritten before return. It therefore needs no field-, tile-, row-, or
column-sized numerical scratch.

For each worker count, the first and settled 100-call allocation phases were:

| Workers | Phase | Allocations | Reallocations | Bytes |
|---:|---|---:|---:|---:|
| 1 | first | 2 | 0 | 3,040 |
| 1 | settled | 1 | 0 | 1,520 |
| 4 | first | 2 | 0 | 3,040 |
| 4 | settled | 1 | 0 | 1,520 |
| 16 | first | 2 | 0 | 3,040 |
| 16 | settled | 1 | 0 | 1,520 |

These are field-size-independent scheduler allocations, at most 0.02
allocation and 30.4 bytes per call in the first measured phase.

## Reproduction

```sh
./scripts/benchmark-omega-diagnosis-fortran.sh
cargo bench -p wrf-dynamics --bench omega_diagnosis -- --noplot
cargo run -p wrf-dynamics --release --example measure_omega_diagnosis_allocations
```

## Scope

This is one isolated routine on one machine. It does not establish whole-model
performance, forecast throughput, or GPU behavior.
