# Randomized differential testing

Focused examples are readable and essential, but they cover only a handful of
shapes and values. The seeded ARW corpus adds breadth without giving up the
strong provenance of a compiled upstream oracle.

## One input, two implementations

The corpus generator writes floating-point inputs as raw IEEE-754 binary32 bits
and records WRF's domain, memory, and tile bounds as integers. Both the pinned
Fortran routine and the Rust kernel read those exact committed inputs. The test
therefore does not depend on two supposedly equivalent random-number generators
or decimal parsing behavior.

The verification flow is:

```text
versioned Rust generator
        |
        v
committed raw-bit inputs -- byte regeneration check
        |                         |
        v                         v
pinned WRF routine          multithreaded Rust kernel
        |                         |
        +------ complete output comparison ------+
```

There are 68 cases and 39,588 complete output values across positive-definite
sheet/slab correction, Held-Suarez damping, and column-mass staggering. The
column-mass set crosses all four west-east boundary states with all four
south-north states. Other cases vary small shapes, clipping, and non-one memory
origins.

## Reproduction and diagnosis

Run:

```sh
./scripts/randomized-arw/run-oracles.sh
cargo test -p wrf-dynamics seeded_randomized -- --nocapture
```

The oracle regenerates the inputs and fails if they differ from the committed
files before compiling Fortran. A Rust divergence reports the seed, field, and
first mismatching linear index. Cases remain deliberately small, so that tuple
is a directly inspectable reproducer rather than a giant opaque fuzz artifact.

## Floating-point policy

Finite outputs, signed zero, and infinities require raw-bit equality. A NaN is
compared by IEEE class because payload and sign propagation can vary across
compilers and devices without conveying useful atmospheric information.

The kernels do not scan whole fields for non-finite inputs; WRF does not do so,
and adding the scan would change timestep cost. Non-finite state is treated as
a diagnostic failure upstream of these kernels. The corpus nevertheless feeds
active NaN and infinity values so their propagation is intentional and visible.

Finite magnitude extremes exposed a separate WRF behavior: the
positive-definite scale multiplication can overflow before its reciprocal is
applied even when the normalized result is representable. That reproducer is
tracked as WRF-008 in the upstream findings ledger, while Rust retains the
Fortran result for compatibility.

## Why this is not fuzzing

The corpus is deterministic, reviewed, and always run in CI. It complements
future property-based and coverage-guided fuzzing, which may explore far more
inputs but usually cannot compile the full Fortran oracle for every mutation.
New fuzz discoveries should be minimized and promoted into this corpus with a
named seed and explicit parity policy.
