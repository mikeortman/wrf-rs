//! Physical parameterization kernels translated from pinned WRF source.
//!
//! Each scheme exposes a narrow backend capability with backend-owned fields
//! and workspace. The standard CPU implementation uses persistent host
//! parallelism; future GPU implementations can keep fields and scratch storage
//! device-resident behind the same scientific operation.

#![forbid(unsafe_code)]

mod microphysics;

pub use microphysics::{
    CpuKesslerMicrophysicsWorkspace, KesslerMicrophysicsAxis, KesslerMicrophysicsError,
    KesslerMicrophysicsField, KesslerMicrophysicsFields, KesslerMicrophysicsKernels,
    KesslerMicrophysicsParameter, KesslerMicrophysicsParameters, KesslerMicrophysicsRegion,
    KesslerMicrophysicsResult, MicrophysicsBoundaryPolicy, MicrophysicsDriver,
    MicrophysicsDriverDomain, MicrophysicsDriverError, MicrophysicsDriverFields,
    MicrophysicsDriverResult, MicrophysicsDriverWorkspace, MicrophysicsScheme, MicrophysicsTile,
    MoistureSpecies, MoistureSpeciesIndex, MoistureSpeciesPackage,
};
