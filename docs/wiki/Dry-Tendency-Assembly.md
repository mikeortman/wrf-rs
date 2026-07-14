# Dry Runge-Kutta tendency assembly

`rk_addtend_dry` builds the dry large-timestep tendencies consumed by an ARW
Runge–Kutta update. It sits after diagnostic preparation and after individual
dynamics tendency terms have been calculated. Its job is composition: combine
the current substep's dynamics tendency with physics and boundary tendencies
that remain fixed across the RK step.

## Two kinds of tendency

For each momentum component, geopotential, and potential temperature, WRF
stores two arrays:

- `*_tend` contains substep-dependent dynamics such as advection and pressure
  gradients;
- `*_tendf` contains forward-step physics and boundary tendencies that are
  assembled on the first RK substep and then reused.

The Rust API names these `RungeKuttaTendencies` and `ForwardTendencies` and
represents the branch as `DryTendencyAssemblyPhase::{FirstSubstep,
LaterSubstep}`. An enum makes the persistence rule explicit; callers cannot
silently assign an undocumented meaning to another integer.

## Equations and map factors

On the first substep, saved boundary tendencies are accumulated into the
persistent field. Then the persistent field is coupled and added to the RK
tendency. In WRF source order:

```text
ruf = ruf + u_save * msfuy       ru = ru + ruf / msfuy
rvf = rvf + v_save * msfvx       rv = rv + rvf * msfvx_inv
rwf = rwf + w_save * msfty       rw = rw + rwf / msfty
phf = phf + ph_save              ph = ph + phf / msfty
tf  = tf  + t_save               t  = t + tf/msfty
                                      + (c1*mut + c2)*h_diabatic/msfty
mu  = mu + muf
```

The first assignments in each row are skipped on later substeps. Column-mass
forward tendency is only read, so Rust borrows it immutably even though the
Fortran dummy argument is declared `INTENT(INOUT)`.

## C-grid ranges

The five volume loops deliberately do not share one rectangular bound:

| Equation | West-east | South-north | Bottom-top |
|---|---|---|---|
| U | full tile | clipped to mass domain | `kts..kte-1` |
| V | clipped | full tile | `kts..kte-1` |
| W and geopotential | clipped | clipped | full `kts..kte` stagger |
| Potential temperature | clipped | clipped | `kts..kte-1` |
| Column mass | clipped | clipped | horizontal |

The region owns physical mass-domain and inclusive-WRF-tile metadata and
derives these ranges. A dedicated interior-tile regression is important:
full-domain clipping can accidentally hide the one-level difference between
`kte-1` and `kte`.

## Safe parallel execution

The CPU backend divides fields into complete contiguous west-east rows. A safe
paired-output primitive hands each worker matching disjoint RK and persistent
rows, so a first-substep point is completed in one memory pass. Shared saved
fields, thermodynamics, coefficients, and map factors are immutable. There is
no unsafe code, numerical scratch, or field clone.

The capability trait retains backend-native field storage. A future GPU backend
can implement the equations as device kernels or fuse components without
accepting CPU closures or copying fields through host memory.

## Parity and validation

The oracle script extracts the exact pinned WRF v4.7.1 routine and compares
5,616 complete mutable and sentinel values. It covers first and later substeps,
all upper staggers, an interior vertical tile, zero and signed-zero map factors,
infinities, and untouched storage. Finite values and infinities require raw-bit
equality; NaNs require class equality. One- and four-worker outputs are
bit-identical, and coefficient failure is proven to leave every mutable field
unchanged.

The matched benchmark finds parallel Rust faster than serial WRF, so no SIMD
specialization is justified yet. See the [detailed performance baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/dry-tendency-assembly-2026-07-14.md).
