//! Acoustic advancement of column mass, vertical mass flux, and potential temperature.
//!
//! This module translates WRF v4.7.1 `advance_mu_t`. The public capability
//! names actual field roles instead of preserving the source routine's dead
//! positional arguments, while the CPU implementation retains WRF operation
//! order and exact single-precision behavior.

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
    AcousticMassThetaBoundaryPolicy, AcousticMassThetaLateralDomain,
    AcousticMassThetaWestEastPeriodicity,
};
pub use coefficient::AcousticMassThetaCoefficient;
pub(crate) use cpu::validate_acoustic_mass_and_theta;
pub use domain::{AcousticMassThetaAxis, AcousticMassThetaRegion};
pub use error::{AcousticMassThetaError, AcousticMassThetaResult};
pub use field::AcousticMassThetaField;
pub use fields::{
    AcousticMassThetaDiagnostics, AcousticMassThetaInputs, AcousticMassThetaMapFactors,
    AcousticMassThetaMassInputs, AcousticMassThetaMomentumInputs, AcousticMassThetaState,
    AcousticMassThetaThermodynamicInputs, AcousticMassThetaVerticalCoefficients,
};
pub use kernels::AcousticMassThetaKernels;
pub use parameters::AcousticMassThetaParameters;
