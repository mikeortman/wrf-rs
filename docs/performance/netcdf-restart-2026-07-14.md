# NetCDF restart I/O baseline — 2026-07-14

## Result

The correctness fixture's control-plane cost is in the same practical class:
1,000 complete tiny restarts took 0.56 seconds through NetCDF-C and 0.44
seconds through Rust in the recorded run. This workload is intentionally too
small for a speedup claim.

Bulk classic-format output is slower in the current safe Rust dependency. On
25 overwrites of one 256 × 256 × 64 single-precision field (400 MiB total),
timing inside the write loop was:

| Writer | Elapsed | Effective payload rate | Relative |
|---|---:|---:|---:|
| NetCDF-C 4.10.1, `-O3` driver | 0.242086 s | 1,652 MiB/s | 1.00× |
| Rust 1.96.0 release, thin LTO | 0.543888 s | 735 MiB/s | 2.25× slower |

Peak resident memory from a separate warmed 25-write run was 28,622,848 bytes
for NetCDF-C and 19,562,496 bytes for Rust. Both include the same 16 MiB caller
field. The Rust process used about 32% less peak resident memory in this
measurement.

This is an in-cache local overwrite benchmark, not forecast throughput. It
does not include NetCDF-4 compression, multiple fields, parallel filesystems,
or model computation. File-cache and close timing vary, so the committed
script reports raw elapsed values rather than presenting confidence intervals
that were not measured.

## Optimization decision

The first pure-Rust attempt exposed every four-byte value directly to the OS
writer. The 16 MiB test did not complete 25 repetitions within 71 seconds.
Wrapping the same seekable file in a one-MiB standard `BufWriter` removed the
syscall pathology while preserving the independent exact-bit oracle. The
accepted implementation remains simple, safe, and bounded.

The remaining bulk gap comes from scalar per-value big-endian conversion and
small writes inside `netcdf3`; NetCDF-C performs that conversion more
efficiently. No local unsafe code or bespoke SIMD serializer was added. I/O is
not yet shown to dominate an end-to-end WRF workload, and the current design
is memory-conservative, so the port records the gap and moves on. A future
change should be evidence-gated against realistic multi-field restarts and
prefer an upstream buffered/bulk-write improvement in the dependency.

## Configuration

- Machine: Apple M3 Max, macOS 26.2 arm64.
- Rust: rustc 1.96.0, workspace release profile (`opt-level=3`, thin LTO, one
  codegen unit).
- C: Apple Clang using `-O3`; NetCDF-C 4.10.1 from Homebrew.
- Format: classic NetCDF 64-bit offset, no compression.
- Bulk field: 4,194,304 `float` values, 16 MiB per write, caller allocation
  reused outside the timed loop.
- Writer buffer: one MiB, allocated once per output file.

## Reproduce

```sh
./scripts/benchmark-netcdf-restart.sh 1000
```

`FIELD_REPETITIONS` changes the default 25 bulk writes.
