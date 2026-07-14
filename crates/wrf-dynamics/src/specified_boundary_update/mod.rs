//! Specified-zone updates and finalization for ARW prognostic fields.
//!
//! This module translates WRF v4.7.1 tendency, geopotential, zero-gradient,
//! and flow-dependent boundary routines. Typed field locations and flow roles
//! replace source selectors, while validated regions own staggered execution
//! ranges and neighbor contracts.

mod cpu;
mod error;
mod field_location;
mod flow_dependent;
pub(crate) mod geometry;
mod geopotential;
mod kernels;
mod parameters;
mod periodicity;
mod region;
mod zero_gradient;

pub use error::{SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateResult};
pub use field_location::SpecifiedBoundaryFieldLocation;
pub use flow_dependent::{
    SpecifiedBoundaryFlowError, SpecifiedBoundaryFlowField, SpecifiedBoundaryFlowInputs,
    SpecifiedBoundaryFlowKernels, SpecifiedBoundaryFlowParameters, SpecifiedBoundaryFlowRegion,
    SpecifiedBoundaryFlowResult,
};
pub use geopotential::{
    SpecifiedBoundaryGeopotentialError, SpecifiedBoundaryGeopotentialInputs,
    SpecifiedBoundaryGeopotentialKernels, SpecifiedBoundaryGeopotentialResult,
};
pub use kernels::SpecifiedBoundaryUpdateKernels;
pub use parameters::SpecifiedBoundaryUpdateParameters;
pub use periodicity::SpecifiedBoundaryWestEastPeriodicity;
pub use region::{SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateRegion};
pub use zero_gradient::{
    SpecifiedBoundaryZeroGradientError, SpecifiedBoundaryZeroGradientKernels,
    SpecifiedBoundaryZeroGradientParameters, SpecifiedBoundaryZeroGradientResult,
};
