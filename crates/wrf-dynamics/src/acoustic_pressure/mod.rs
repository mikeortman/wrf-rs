//! Pressure, inverse-density, and hydrostatic geopotential updates for acoustic steps.
//!
//! This module translates WRF v4.7.1 `calc_p_rho`. Its typed boundary keeps
//! prognostic state, thermodynamics, column mass, vertical metrics, execution
//! mode, and divergence-damping phase distinct while preserving backend-native
//! storage for a future GPU implementation.

mod axis;
mod coefficient;
mod coefficients;
mod cpu;
mod damping_phase;
mod error;
mod field;
mod kernels;
mod masses;
mod mode;
#[cfg(test)]
mod oracle_tests;
mod parameters;
mod region;
mod state;
mod thermodynamics;
mod vertical_metrics;

pub use axis::AcousticPressureAxis;
pub use coefficient::AcousticPressureCoefficient;
pub use coefficients::AcousticPressureCoefficients;
pub(crate) use cpu::validate_acoustic_pressure;
pub use damping_phase::AcousticPressureDampingPhase;
pub use error::{AcousticPressureError, AcousticPressureResult};
pub use field::AcousticPressureField;
pub use kernels::AcousticPressureKernels;
pub use masses::AcousticPressureMasses;
pub use mode::AcousticPressureMode;
pub use parameters::AcousticPressureParameters;
pub use region::AcousticPressureRegion;
pub use state::AcousticPressureState;
pub use thermodynamics::AcousticPressureThermodynamics;
pub use vertical_metrics::AcousticPressureVerticalMetrics;
