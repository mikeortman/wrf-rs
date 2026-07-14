//! Prognostic time-level and coupled-perturbation preparation for ARW acoustic steps.
//!
//! This module translates WRF v4.7.1 `small_step_prep`. The safe Rust API
//! groups the routine's positional arrays by ownership and scientific role,
//! while retaining backend-native storage for a future GPU implementation.

mod axis;
mod coefficient;
mod coefficients;
mod column_mass_time_levels;
mod cpu;
mod diagnostic_inputs;
mod error;
mod field;
mod kernels;
mod map_factors;
mod mass_inputs;
mod mass_outputs;
#[cfg(test)]
mod oracle_tests;
mod phase;
mod region;
mod saved_outputs;
mod volume_time_levels;

pub use axis::AcousticStepPreparationAxis;
pub use coefficient::AcousticStepPreparationCoefficient;
pub use coefficients::AcousticStepPreparationCoefficients;
pub use column_mass_time_levels::AcousticStepPreparationColumnMassTimeLevels;
pub use diagnostic_inputs::AcousticStepPreparationDiagnosticInputs;
pub use error::{AcousticStepPreparationError, AcousticStepPreparationResult};
pub use field::AcousticStepPreparationField;
pub use kernels::AcousticStepPreparationKernels;
pub use map_factors::AcousticStepPreparationMapFactors;
pub use mass_inputs::AcousticStepPreparationMassInputs;
pub use mass_outputs::AcousticStepPreparationMassOutputs;
pub use phase::AcousticStepPreparationPhase;
pub use region::AcousticStepPreparationRegion;
pub use saved_outputs::AcousticStepPreparationSavedOutputs;
pub use volume_time_levels::AcousticStepPreparationVolumeTimeLevels;
