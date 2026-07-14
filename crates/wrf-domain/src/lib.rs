//! Domain decomposition, tile bounds, and halo exchange for WRF grids.
//!
//! WRF exposes inclusive Fortran bounds such as `ids:ide`. This crate converts
//! them once at the boundary into signed, zero-based, half-open [`IndexRange`]
//! values. Signed indices are intentional: periodic and physical-boundary
//! halos can live outside the physical domain.
//!
//! [`DomainTopology`] reproduces RSL_LITE's centered-remainder decomposition.
//! [`HaloExchangePlan`] is storage- and transport-neutral; the included
//! [`LocalHaloExchange`] provides a deterministic reference executor.

#![forbid(unsafe_code)]

mod bounds;
mod exchange;
mod topology;

pub use bounds::{
    DomainBounds, HorizontalBounds, IndexRange, MemoryBounds, PatchBounds, TileBounds,
};
pub use exchange::{
    ExchangeAxis, ExchangeDirection, HaloExchangeError, HaloExchangePlan, HaloExchangeResult,
    HaloTransfer, HorizontalPeriodicity, HorizontalStaggering, LocalHaloExchange, LocalPatchField,
    PatchField,
};
pub use topology::{
    BoundaryWidths, DomainTopology, PatchCoordinate, PatchId, ProcessGrid, TileGrid, TopologyError,
    TopologyResult,
};
