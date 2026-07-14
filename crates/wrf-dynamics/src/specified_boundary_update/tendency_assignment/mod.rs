//! Assignment of boundary-file tendencies into WRF's specified zone.

mod boundary_tendencies;
mod cpu;
mod error;
mod kernels;
mod parameters;

pub(crate) use cpu::validate_cpu_tendency_assignment;

pub use boundary_tendencies::SpecifiedBoundaryTendencies;
pub use error::{SpecifiedBoundaryTendencyError, SpecifiedBoundaryTendencyResult};
pub use kernels::SpecifiedBoundaryTendencyKernels;
pub use parameters::SpecifiedBoundaryTendencyParameters;
