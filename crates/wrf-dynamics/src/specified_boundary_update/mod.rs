//! Specified-zone updates and finalization for ARW prognostic fields.
//!
//! This module translates WRF v4.7.1 tendency advancement, boundary-file
//! tendency assignment, geopotential, zero-gradient, and flow-dependent
//! routines. Typed field locations and flow roles replace source selectors,
//! while validated regions own staggered execution ranges and neighbor
//! contracts.

mod cpu;
mod dry_relaxation;
mod error;
mod field_location;
mod finalization;
mod flow_dependent;
pub(crate) mod geometry;
mod geopotential;
mod kernels;
mod parameters;
mod periodicity;
mod region;
mod relaxation;
mod tendency_assignment;
mod zero_gradient;

pub use dry_relaxation::{
    DryBoundaryRelaxationBoundaryData, DryBoundaryRelaxationBoundaryFields,
    DryBoundaryRelaxationError, DryBoundaryRelaxationField, DryBoundaryRelaxationKernels,
    DryBoundaryRelaxationMassCoefficients, DryBoundaryRelaxationRegion,
    DryBoundaryRelaxationResult, DryBoundaryRelaxationState, DryBoundaryRelaxationTarget,
    DryBoundaryRelaxationTendencies, DryBoundaryRelaxationWorkspace, DryBoundaryVerticalRelaxation,
};
pub use error::{SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateResult};
pub use field_location::SpecifiedBoundaryFieldLocation;
pub use finalization::{
    SpecifiedBoundaryFinalizationBoundaryFields, SpecifiedBoundaryFinalizationError,
    SpecifiedBoundaryFinalizationFieldLocation, SpecifiedBoundaryFinalizationInputs,
    SpecifiedBoundaryFinalizationKernels, SpecifiedBoundaryFinalizationParameters,
    SpecifiedBoundaryFinalizationRegion, SpecifiedBoundaryFinalizationResult,
};
pub use flow_dependent::{
    SpecifiedBoundaryFlowError, SpecifiedBoundaryFlowField, SpecifiedBoundaryFlowInputs,
    SpecifiedBoundaryFlowKernels, SpecifiedBoundaryFlowParameters, SpecifiedBoundaryFlowRegion,
    SpecifiedBoundaryFlowResult, SpecifiedBoundaryInflowPolicy,
};
pub use geopotential::{
    SpecifiedBoundaryGeopotentialError, SpecifiedBoundaryGeopotentialInputs,
    SpecifiedBoundaryGeopotentialKernels, SpecifiedBoundaryGeopotentialResult,
};
pub use kernels::SpecifiedBoundaryUpdateKernels;
pub use parameters::SpecifiedBoundaryUpdateParameters;
pub use periodicity::SpecifiedBoundaryWestEastPeriodicity;
pub use region::{SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateRegion};
pub use relaxation::{
    SpecifiedBoundaryRelaxationBoundaryValues, SpecifiedBoundaryRelaxationCoefficients,
    SpecifiedBoundaryRelaxationError, SpecifiedBoundaryRelaxationField,
    SpecifiedBoundaryRelaxationInputs, SpecifiedBoundaryRelaxationKernels,
    SpecifiedBoundaryRelaxationParameters, SpecifiedBoundaryRelaxationResult,
};
pub use tendency_assignment::{
    SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyError, SpecifiedBoundaryTendencyKernels,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryTendencyResult,
};
pub use zero_gradient::{
    SpecifiedBoundaryZeroGradientError, SpecifiedBoundaryZeroGradientKernels,
    SpecifiedBoundaryZeroGradientParameters, SpecifiedBoundaryZeroGradientResult,
};
