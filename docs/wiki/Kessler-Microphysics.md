# Kessler warm-rain microphysics

Kessler microphysics is a bulk warm-cloud parameterization. It represents
water in three prognostic categories—vapor, suspended cloud liquid, and falling
rain—without ice-phase species. WRF's implementation also updates potential
temperature through latent heating and diagnoses surface precipitation.

## Place in WRF

The pinned implementation is `phys/module_mp_kessler.F`. It is a complete
microphysics option invoked by WRF's physics driver. The Rust port implements
the scheme routine itself; driver selection and full model-step coupling remain
future work.

Kessler is the first `wrf-physics` capability because it is scientifically
meaningful, self-contained, horizontally parallel, and small enough for a
complete direct oracle. The selection inventory is maintained in
[`docs/physics/kessler-selection.md`](https://github.com/mikeortman/wrf-rs/blob/main/docs/physics/kessler-selection.md).

## State variables

For each grid point, the scheme reads or modifies:

- potential temperature `t`;
- water-vapor mixing ratio `qv`;
- cloud-water mixing ratio `qc`;
- rain-water mixing ratio `qr`;
- dry-air density `rho`;
- Exner function `pii`;
- mass-level height `z`; and
- W-level layer thickness `dz8w`.

At each horizontal point it also updates accumulated non-convective
precipitation `RAINNC` and the current-call precipitation `RAINNCV`.

## Algorithm

### 1. Terminal velocity

Rain terminal velocity depends on rain mass and a density correction:

\[
v_t = 36.34\,(\max(0, 10^{-3}\rho q_r))^{0.1364}
      \sqrt{\frac{\rho_1}{\rho_k}}.
\]

The fall Courant number is evaluated against layer thickness. If the full
physics time step is too large, sedimentation is split into stable substeps.
The number of remaining substeps may be recalculated after each fallout update
because terminal velocity changes with rain content.

### 2. Upwind sedimentation

Rain is transported downward with a first-order upstream flux. The bottom
outgoing flux becomes surface precipitation. Interior levels use density-
weighted flux differences; the top level has no incoming flux from above.

This phase is vertically coupled. Levels in one column must remain ordered,
but horizontal columns do not depend on each other.

### 3. Cloud-to-rain conversion

Cloud water becomes rain through two processes:

- autoconversion above a cloud-water threshold; and
- accretion of cloud droplets by existing rain.

Both processes are limited so cloud water remains nonnegative.

### 4. Saturation adjustment

Temperature and Exner function determine pressure and saturation vapor
pressure. Supersaturated vapor condenses into cloud water; subsaturated air may
evaporate rain. Latent heating updates potential temperature. Vapor, cloud, and
rain limits preserve nonnegative species for the finite atmospheric inputs
covered by the oracle.

## Rust ownership and layout

`KesslerMicrophysicsFields` borrows ten backend-native fields. It owns nothing
and performs no clones. `KesslerMicrophysicsRegion` validates allocation shape
and active half-open ranges before mutation. The pinned routine assumes the
active vertical range begins at the surface and needs at least two levels; the
Rust type makes both conditions explicit.

`CpuKesslerMicrophysicsWorkspace` owns:

- one production scratch value per three-dimensional grid point; and
- one vertical terminal-velocity buffer per persistent CPU worker.

Workspace construction is separate from timestep execution. This removes
field-sized call-time allocation while keeping scratch ownership explicit.

## Parallel execution

Fields use WRF-compatible `(i,k,j)` logical order with west-east points
contiguous. The CPU implementation partitions independent south-north rows
across the default persistent Rayon pool. Within a row, columns retain WRF's
vertical operation ordering.

The capability trait exposes backend-owned `Field` and `Workspace` associated
types. It does not expose host slices, Rayon, or closures, so a future GPU
backend can provide device-native storage and kernels without changing the
scientific call.

## Parity evidence

`scripts/run-kessler-oracle.sh` compiles the pinned WRF module directly. The
fixture includes:

- dry and moist columns;
- zero, light, moderate, and heavy rain;
- cloud water below and above the autoconversion threshold;
- multi-substep sedimentation;
- non-one horizontal memory origins in Fortran;
- excluded horizontal halos; and
- initialized accumulated and step precipitation.

All 660 mutable outputs match by raw IEEE-754 bits. Rust tests also prove
one-worker/four-worker determinism, validation before mutation, parameter
rejection, vertical-contract enforcement, and unchanged halos.

## Performance

On the recorded 655,360-point workload, one-worker Rust and optimized serial
Fortran are within about three percent. Four-worker Rust is 3.56× faster than
serial Fortran and 16-worker Rust is 6.34× faster. The implementation therefore
keeps its readable scalar arithmetic; no speculative SIMD layer is present.

See the [detailed baseline](https://github.com/mikeortman/wrf-rs/blob/main/docs/performance/kessler-microphysics-2026-07-13.md).

## Remaining integration work

Routine parity does not yet prove forecast parity. Remaining work includes the
microphysics driver, Registry moisture-species mapping, ARW tendency coupling,
halo scheduling around physics, restart state, and idealized precipitation
trajectories.
