# Momentum coupling

WRF's prognostic velocity components are converted into mass-coupled momentum
before several ARW transport and tendency calculations. In `rk_step_prep`, this
operation follows the calculation of full column mass at mass, west-east, and
south-north points. The pinned routine is `couple_momentum` in
`dyn_em/module_big_step_utilities_em.F`.

## C-grid quantities

The three velocity components occupy different locations on the Arakawa C
grid:

- `u` and its output `ru` lie on west-east momentum faces;
- `v` and `rv` lie on south-north momentum faces; and
- `w` and `rw` lie on vertically staggered full levels at horizontal mass
  points.

The inputs `muu`, `muv`, and `mut` are full dry-air column mass at those
horizontal locations. `c1h` and `c2h` are half-level coefficients used for
horizontal momentum. `c1f` and `c2f` are the full-level counterparts used for
vertical momentum.

## Equations and operation order

At each active west-east momentum point, WRF computes

```text
ru(i,k,j) = u(i,k,j) * (c1h(k) * muu(i,j) + c2h(k)) / msfu(i,j)
```

South-north momentum uses the inverse map factor supplied by the caller:

```text
rv(i,k,j) = v(i,k,j) * (c1h(k) * muv(i,j) + c2h(k)) * msfv_inv(i,j)
```

Vertical momentum uses mass-point column mass and the full-level coefficients:

```text
rw(i,k,j) = w(i,k,j) * (c1f(k) * mut(i,j) + c2f(k)) / msft(i,j)
```

Rust retains these exact single-precision multiplication, addition, and final
division/multiplication sequences. It does not precompute a combined scale or
replace division with a reciprocal, because either change can alter output
bits, infinities, and signed zero.

## Stagger-specific clipping

WRF receives one inclusive tile for all components, then clips only axes on
which a component is not staggered. In zero-based half-open Rust ranges:

| Output | West-east range | South-north range | Bottom-top range |
|---|---|---|---|
| `ru` | complete tile, including upper U face | clipped to mass domain | clipped to half levels |
| `rv` | clipped to mass domain | complete tile, including upper V face | clipped to half levels |
| `rw` | clipped to mass domain | clipped to mass domain | complete tile, including upper W face |

`MomentumCouplingRegion` stores the physical mass-domain and tile ranges once
and derives these three active boxes. The constructor accepts a tile point on
each upper stagger but rejects empty ranges, storage overflow, or points beyond
that single stagger. Interior subdomain tiles and non-one Fortran memory origins
therefore map to ordinary memory offsets without weakening the physical-domain
contract.

## Rust ownership boundary

The public capability groups fields by scientific role:

- `MomentumCouplingOutputs` owns the three mutable borrows;
- `MomentumCouplingVelocities` owns three immutable velocity borrows;
- `MomentumCouplingMasses` owns the staggered and mass-point column masses;
- `MomentumCouplingMapFactors` owns exactly the three factors used by the
  equations; and
- `MomentumCouplingCoefficients` borrows the four vertical arrays.

Every three-dimensional field must match the region shape. Every mass and map
factor field must match its horizontal shape, and every coefficient array must
span the allocated vertical extent. All checks finish before any output is
mutated. This avoids domain-sized clones while giving a future GPU backend a
narrow native-kernel interface rather than a CPU closure.

The Rust API intentionally omits WRF's `msfv` parameter. The pinned routine
declares and receives it but never reads it; only `msfv_inv` participates in
the south-north equation.

## Parallel execution and memory

Each output pass schedules disjoint west-east rows on the persistent default
CPU pool. Inputs are shared immutably and no numerical scratch is allocated.
The accepted hot loop slices each active input and output row to equal lengths,
then traverses those slices with safe iterators. This both states the ownership
relationship clearly and lets LLVM eliminate per-element bounds checks.

At 1, 4, and 16 workers, two warmed 100-call phases each recorded five small
scheduler allocations totaling 7,600 bytes and no reallocations. There is no
field-, row-, or column-sized temporary storage.

## Parity evidence

`scripts/run-momentum-coupling-oracle.sh` extracts the exact pinned routine and
compiles it without modifying the scientific body. Six cases cover an interior
tile, independent upper west-east/south-north/bottom-top clipping, all upper
staggers together, non-one and negative Fortran memory origins, untouched
storage, finite overflow, division by zero, and multiplication by a zero
inverse map factor.

The golden file stores all 3,780 output and sentinel values as raw IEEE-754
bits. Rust matches every value. Separate tests verify one/four-worker bitwise
determinism, every coefficient role, typed region failures, and validation
before mutation.

WRF contains no dedicated numerical regression for this routine. A randomized
corpus and an end-to-end `rk_step_prep` trajectory remain later gates.

## Performance

On the matched 256 × 256 × 40 workload, optimized serial Fortran measured a
1.152625 ms median. Rust measured 1.3679 ms with one worker and 654.95 µs with
four. Serial Rust is 18.7% slower; standard four-worker Rust is 1.76× faster
than serial Fortran. Sixteen workers are slower than four on this streaming
workload.

The full raw samples, confidence intervals, allocation evidence, and rejected
global-index loop are recorded in the
[performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/momentum-coupling-2026-07-13.md).
