# Parity and testing

## Observable parity

The port seeks equivalent outputs and failure behavior, not identical source.
Parity has several scales:

- **discrete parity** covers dimensions, configuration, status values, names,
  metadata, and exact rational values;
- **kernel parity** compares isolated numerical routines on identical inputs;
- **trajectory parity** compares intermediate fields and norms by timestep;
- **operational parity** covers initialization, nesting, distributed execution,
  restart, and scientific I/O.

A late forecast that falls inside a loose tolerance can conceal an early
algorithmic divergence. Trajectory checkpoints therefore remain necessary even
after end-to-end cases run.

## Oracle construction

A differential oracle compiles the pinned upstream Fortran routine itself. A
small driver supplies deterministic inputs and emits a machine-independent
representation where practical. Single-precision exact fixtures use eight hex
digits per IEEE-754 bit pattern, preserving signed zero and avoiding decimal
format ambiguity.

The golden output is committed and consumed by Rust tests. This creates one
chain of provenance:

`pinned Fortran source -> oracle driver -> golden bits -> Rust assertion`.

Hand-calculated expected values are useful for review but do not replace that
chain. During the first dynamics slice, the oracle immediately corrected an
incorrect hand-derived expectation.

## Added tests

Upstream tests are a floor. Rust tests add missing branch boundaries, typed
shape failures, signed zero, concurrency determinism, worker failure mapping,
and numerical invariants. The current ARW randomized corpora use committed
seeds and shared raw-bit inputs; failures print the seed, field, and first
divergent index. See [Randomized differential
testing](Randomized-Differential-Testing.md).

Exact equality is preferred whenever operation order can be preserved. When a
parallel reduction, device implementation, or different elementary function
library makes exact equality inappropriate, the algorithm page must justify a
comparison policy and tests must report absolute, relative, and ULP error.
