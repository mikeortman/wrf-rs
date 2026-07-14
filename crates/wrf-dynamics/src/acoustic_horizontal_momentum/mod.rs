//! Explicit horizontal-momentum advancement during WRF acoustic steps.
//!
//! This module translates WRF v4.7.1 `advance_uv`. Typed field groups and
//! boundary enums replace the source routine's long positional interface while
//! retaining backend-native storage for a future GPU implementation.

mod boundary;
mod coefficient;
mod cpu;
mod domain;
mod error;
mod field;
mod fields;
mod kernels;
mod parameters;

pub use boundary::{
    AcousticHorizontalBoundaryPolicy, AcousticRelaxationZone, AcousticSouthNorthBoundary,
    AcousticWestEastBoundary, AcousticWestEastPeriodicity,
};
pub use coefficient::AcousticHorizontalMomentumCoefficient;
pub use domain::{AcousticHorizontalMomentumAxis, AcousticHorizontalMomentumRegion};
pub use error::{AcousticHorizontalMomentumError, AcousticHorizontalMomentumResult};
pub use field::AcousticHorizontalMomentumField;
pub use fields::{
    AcousticHorizontalMapFactors, AcousticHorizontalMassFields,
    AcousticHorizontalMoistureCoefficients, AcousticHorizontalMomentumInputs,
    AcousticHorizontalMomentumState, AcousticHorizontalMomentumTendencies,
    AcousticHorizontalPressureFields, AcousticHorizontalVerticalCoefficients,
};
pub use kernels::AcousticHorizontalMomentumKernels;
pub use parameters::AcousticHorizontalMomentumParameters;
