//! Implicit acoustic advancement of vertical momentum and geopotential.
//!
//! This module translates WRF v4.7.1 `advance_w`. The backend capability
//! exposes one reusable right-hand-side workspace because the implicit solve
//! genuinely needs the old geopotential and vertical momentum at the same
//! time. The CPU implementation performs no field-sized timestep allocation.

mod advection;
mod axis;
mod boundary;
mod coefficient;
mod cpu;
mod damping;
mod error;
mod field;
mod fields;
mod kernels;
mod parameters;
mod region;

pub use advection::AcousticVerticalAdvection;
pub use axis::AcousticVerticalAxis;
pub use boundary::{
    AcousticVerticalBoundaryPolicy, AcousticVerticalLateralDomain,
    AcousticVerticalWestEastPeriodicity,
};
pub use coefficient::AcousticVerticalCoefficient;
pub(crate) use cpu::validate_acoustic_vertical_momentum;
pub use damping::AcousticVerticalDamping;
pub use error::{AcousticVerticalError, AcousticVerticalResult};
pub use field::AcousticVerticalField;
pub use fields::{
    AcousticVerticalGeopotentialInputs, AcousticVerticalInputs, AcousticVerticalLevelCoefficients,
    AcousticVerticalMapFactors, AcousticVerticalMassInputs, AcousticVerticalMomentumInputs,
    AcousticVerticalSolveInputs, AcousticVerticalState, AcousticVerticalThermodynamicInputs,
    AcousticVerticalWorkspace,
};
pub use kernels::AcousticVerticalKernels;
pub use parameters::AcousticVerticalParameters;
pub use region::AcousticVerticalRegion;
