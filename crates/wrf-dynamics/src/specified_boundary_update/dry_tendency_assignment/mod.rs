//! Dry-field orchestration for WRF specified-boundary tendency assignment.

mod boundary_fields;
mod cpu;
mod error;
mod kernels;
mod region;
mod target;
mod tendencies;
mod vertical;

pub(crate) use cpu::validate_cpu_dry_boundary_tendency_assignment;

pub use boundary_fields::DryBoundaryTendencyBoundaryFields;
pub use error::{DryBoundaryTendencyError, DryBoundaryTendencyResult};
pub use kernels::DryBoundaryTendencyKernels;
pub use region::DryBoundaryTendencyRegion;
pub use target::DryBoundaryTendencyTarget;
pub use tendencies::DryBoundaryTendencies;
pub use vertical::DryBoundaryVerticalTendency;
