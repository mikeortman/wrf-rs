//! Time-averaged mass-flux accumulation across acoustic substeps.
//!
//! This module translates WRF v4.7.1 `sumflux`. The Rust API omits source
//! arguments that the routine never reads and names each C-grid field role.

mod coefficient;
mod cpu;
mod error;
mod field;
mod fields;
mod kernels;
mod phase;
mod region;

pub use coefficient::AcousticFluxCoefficient;
pub use error::{AcousticFluxAccumulationError, AcousticFluxAccumulationResult};
pub use field::AcousticFluxField;
pub use fields::{
    AcousticFluxCurrentFields, AcousticFluxLinearFields, AcousticFluxMapFactors,
    AcousticFluxMassFields, AcousticFluxRunningAverages,
};
pub use kernels::AcousticFluxAccumulationKernels;
pub use phase::AcousticSubstepPhase;
pub use region::AcousticFluxAccumulationRegion;
