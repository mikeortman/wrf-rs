//! Complete local acoustic trajectory with lateral boundary updates.
//!
//! This capability composes the pinned WRF v4.7.1 `solve_em.F` window from
//! `small_step_prep` through each substep's final physical-boundary block.
//! Generated MPI/periodic halo descriptors and polar filtering remain outside
//! this single-rank local-memory stage.

mod controls;
mod cpu;
mod error;
mod kernels;
mod region_role;
mod regions;

pub use controls::AcousticBoundaryStageControls;
pub use error::{AcousticBoundaryStageError, AcousticBoundaryStageResult};
pub use kernels::AcousticBoundaryStageKernels;
pub use region_role::AcousticBoundaryRegionRole;
pub use regions::AcousticBoundaryStageRegions;
