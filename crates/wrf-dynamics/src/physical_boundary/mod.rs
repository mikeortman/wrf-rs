//! WRF physical boundary-zone assignment (`set_physical_bc3d`/`set_physical_bc2d`).
//!
//! The kernels copy interior values into halo storage for periodic, symmetric,
//! and open/specified/nested lateral boundaries on a single-rank patch. The
//! composed acoustic boundary stage calls them between acoustic substeps in
//! pinned `solve_em.F` order.

mod conditions;
mod cpu;
mod error;
mod kernels;
mod region;
mod variable;

pub use conditions::PhysicalBoundaryConditions;
pub(crate) use cpu::{validate_horizontal_boundary, validate_volume_boundary};
pub use error::{PhysicalBoundaryError, PhysicalBoundaryResult};
pub use kernels::PhysicalBoundaryKernels;
pub use region::{PhysicalBoundaryAxis, PhysicalBoundaryRegion};
pub use variable::PhysicalBoundaryVariable;

/// Fixed WRF boundary-zone depth from `share/module_bc.F` (`bdyzone = 4`).
pub const PHYSICAL_BOUNDARY_ZONE: usize = 4;
