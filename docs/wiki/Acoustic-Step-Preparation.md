# Acoustic small-step preparation

WRF's Advanced Research WRF (ARW) dynamical core advances slow processes on a
large Runge-Kutta time step and compressible acoustic modes on smaller nested
steps. `small_step_prep` is the boundary between those two integrations. It
switches or differences prognostic time levels, saves values needed after the
acoustic loop, and converts velocity and thermodynamic variables into the
mass-coupled perturbations advanced by the small steps.

The Rust implementation translates WRF v4.7.1
`dyn_em/module_small_step_em.F:16-290`. Its public capability is
`AcousticStepPreparationKernels`; the production CPU implementation lives in
`wrf_dynamics::acoustic_step_preparation::cpu`, split into horizontal mass,
volume-field, and validation modules.

## Position in an ARW time step

`solve_em` calls this routine after `rk_step_prep` has formed diagnostic fields
and `rk_addtend_dry` has assembled the dry large-step tendencies. The prepared
state then feeds pressure/density coefficient construction and the acoustic
advance. It is therefore the first ported boundary that transforms the
large-step trajectory into acoustic perturbation variables.

The routine has two modes:

- **First substep.** The current (`*_2`) large-step state replaces the previous
  (`*_1`) state. The perturbation column mass is reset to zero.
- **Later substep.** The previous state remains fixed and the current state is
  replaced by its difference from that reference.

`AcousticStepPreparationPhase` makes this branch explicit instead of exposing
WRF's integer `rk_step` convention.

## Column mass preparation

Let `MUB` be base column mass, `MU_1` the reference perturbation mass, `MU_2`
the current perturbation mass, and `MUT` the full mass used by the large-step
state.

On the first substep, the routine performs

```text
MU_1    = MU_2
MUTS    = MUB + MU_2
MUUS    = MUU
MUVS    = MUV
MU_SAVE = MU_2
MU_2    = 0
MUDF    = 0
```

On later substeps it performs

```text
MUTS(i,j) = MUB(i,j) + MU_1(i,j)
MUUS(i,j) = 0.5 * (MUB(i,j) + MU_1(i,j)
                 + MUB(i-1,j) + MU_1(i-1,j))
MUVS(i,j) = 0.5 * (MUB(i,j) + MU_1(i,j)
                 + MUB(i,j-1) + MU_1(i,j-1))
MU_SAVE   = MU_2
MU_2      = MU_1 - MU_2
```

The addition order is preserved exactly because reassociation can change
single-precision results. Later substeps require valid west and south halo
neighbors; the typed region rejects a tile that cannot provide them.

## Coupled perturbation variables

For half-level coefficient arrays `c1h`, `c2h`, full-level arrays `c1f`,
`c2f`, and map factors, WRF constructs:

```text
U'' = ((c1h*MUUS + c2h)*U_1 - (c1h*MUU + c2h)*U_2) / MSFUY
V'' = ((c1h*MUVS + c2h)*V_1 - (c1h*MUV + c2h)*V_2) * MSFVX_INV
T'' =  (c1h*MUTS + c2h)*T_1 - (c1h*MUT + c2h)*T_2
W'' = ((c1f*MUTS + c2f)*W_1 - (c1f*MUT + c2f)*W_2) / MSFTY
PH'' = PH_1 - PH_2
```

Before replacement, each current field is copied to its corresponding saved
field. The pressure coefficient is

```text
C2A = (cp / cv) * (PB + P) / ALT
```

with WRF single-precision `cp/cv = 1.4`. Diagnosed omega is copied to
`WW_SAVE` across the complete full-level column.

## Grid staggering and range contracts

The API distinguishes the mass-domain range from the tile ranges that may
contain an upper C-grid stagger:

| Quantity | Horizontal extent | Vertical extent |
|---|---|---|
| mass, temperature, pressure coefficient | mass X × mass Y | half levels |
| U and `MUUS` | upper-staggered X × mass Y | half levels for U |
| V and `MUVS` | mass X × upper-staggered Y | half levels for V |
| W, geopotential, omega | mass X × mass Y | half levels plus one full level |

The production WRF caller horizontally tiles this work but always supplies a
complete acoustic column. Rust records that assumption as a checked invariant;
partial vertical tiles fail before mutation.

## Safe ownership and execution

Borrowed role types group the original routine's 68 positional arguments into
time levels, saved outputs, column-mass inputs and outputs, diagnostics, map
factors, coefficients, phase, and region. Mutable aliases cannot be created
through the safe API. Every shape, coefficient length, range, full-column
requirement, and later-step neighbor is validated before any output changes.

The CPU backend runs independent contiguous X lines through its persistent
Rayon pool. Paired saved/current fields are updated in one pass without
numerical scratch or field clones. This is a CPU implementation behind a
storage-associated capability trait, not a CPU closure embedded in the public
contract, so a future GPU backend can provide native dispatch and storage.

## Intentional differences from the Fortran interface

The Rust API omits coefficient arrays `c3h`, `c4h`, `c3f`, and `c4f`; map
arrays `msfux`, `msfvx`, `msfvy`, and `msftx`; inverse grid spacing `rdx` and
`rdy`; and patch bounds that the routine never reads.

WRF also writes zero to two `WW_SAVE` boundary planes and then overwrites every
element of those planes before returning. Rust omits these dead intermediate
stores. The exact oracle proves that the observable result is unchanged.

## Parity and failure evidence

The oracle script extracts and compiles the exact pinned routine body rather
than maintaining a rewritten Fortran reference. Three fixtures cover the first
substep with both upper staggers, a later interior tile with west/south
neighbors, and IEEE exceptional inputs. All 9,936 mutable and inactive-sentinel
records match: finite values and infinities by raw bits, NaNs by class.

Rust tests additionally prove one-versus-four-worker bit identity, full-column
and range validation, and failure atomicity across all 24 mutable fields. The
matched benchmark and allocation evidence are recorded in
`docs/performance/acoustic-step-preparation-2026-07-14.md`.

## Remaining integration work

Routine-level parity does not yet prove an acoustic trajectory. The next
scientific gate is to port the dependent coefficient construction and acoustic
advance routines, bind these role types to Registry-generated state, and compare
the complete small-step state after each acoustic substep and restart.
