# Performance and memory principles

These rules apply to the CPU reference port and constrain future GPU work.
Readability remains a design requirement; performance changes need evidence.

## Hot-path rules

- Allocate domain fields, tendency buffers, and scratch storage during setup.
  Do not allocate inside grid-point loops or routine timestep execution.
- Store numerical fields as contiguous scalar arrays. Prefer structure of
  arrays when values are consumed as separate WRF fields.
- Match WRF's declared precision. Do not silently promote every field to `f64`;
  doing so changes both memory pressure and floating-point behavior.
- Partition outer dimensions or contiguous slabs. Keep inner loops contiguous
  so release builds can auto-vectorize them.
- Use the persistent CPU work-stealing pool. Do not spawn threads per kernel or
  hide nested thread pools inside physics schemes.
- Keep synchronization outside inner loops. Workers receive disjoint mutable
  outputs and shared immutable inputs.
- Vectorize inside worker chunks. Runtime instruction-set dispatch occurs once
  per kernel, with scalar cleanup for tails and no per-point branching.
- Preserve operation order until parity is proven. Reassociation, fused
  operations, and reduction ordering are numerical changes, not free cleanup.

## Ownership and readability

Cloning a small descriptor, typed index, shape, `Arc`, or other lightweight
owner is acceptable when it makes ownership obvious and avoids lifetime-heavy
APIs. Cloning field arrays, tendency buffers, lookup tables, or per-domain state
in a timestep path is not acceptable without profiling evidence and a stated
reason.

## Release configuration

The workspace release profile uses thin LTO and one codegen unit. Any further
flags must be benchmarked on representative WRF kernels and must not assume a
CPU instruction set that breaks the intended deployment target.

## Evidence gates

Every substantial numerical slice should eventually include:

1. an upstream parity fixture;
2. a release-mode benchmark with fixed dimensions and worker count;
3. allocation measurements for setup and steady-state execution;
4. scaling measurements across worker counts;
5. a profile showing the actual hot loops before micro-optimization.

An optimization is accepted only when parity remains within its documented
policy and the benchmark shows a meaningful gain.

## Current benchmark corpus

`cargo bench -p wrf-dynamics --bench positive_definite` uses Criterion with a
1,048,576-value field, all lines requiring correction, and worker counts of
one, four when available, and all host workers. Input cloning occurs in
Criterion's excluded batch setup so the measured interval contains the kernel,
not fixture restoration. Sheet and slab variants report element throughput.

Use `-- --quick` for a development smoke run. Saved performance claims must use
the normal statistical run, record CPU/OS/toolchain details, and retain the raw
Criterion output or machine-readable artifact.

`cargo run -p wrf-dynamics --release --example
measure_positive_definite_allocations` measures two warmed 100-dispatch phases
with an instrumented system allocator. It enforces no reallocations, fewer than
one small allocation per ten calls, and at most 64 KiB allocated per phase.
These are scheduler budgets, not permission for numerical scratch allocation.

`cargo bench -p wrf-dynamics --bench held_suarez` measures 2,097,152
momentum-tendency updates over two staggered components. It retains six
domain-sized fields but restores only the two outputs in excluded setup. The
2026-07-13 Apple M3 Max baseline found four workers faster than both one and all
16 host workers; see `docs/performance/held-suarez-2026-07-13.md`.
