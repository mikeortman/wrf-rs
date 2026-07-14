//! Numerical kernels from WRF's Advanced Research WRF dynamical core.
//!
//! Each kernel family exposes a focused backend capability so CPU and future
//! GPU implementations can use native storage and execution strategies.
//! [`RungeKuttaPreparationKernels`] composes the seven translated ARW
//! diagnostics behind one failure-atomic validation boundary.
//!
//! The crate preserves WRF's observable numerical behavior, not its Fortran
//! implementation structure. Safe in-place mutation, persistent parallelism,
//! and typed shape checks replace temporary arrays and implicit contracts when
//! those changes retain parity.
//!
//! Focused fixtures and seeded randomized corpora compile the pinned WRF
//! routines and compare complete single-precision outputs. Finite values,
//! signed zero, and infinities require raw-bit equality; NaN requires class
//! equality because its payload is not a portable atmospheric data contract.

#![forbid(unsafe_code)]

mod column_mass_staggering;
mod held_suarez;
mod inverse_density;
mod moisture_coefficients;
mod momentum_coupling;
mod omega_diagnosis;
mod positive_definite;
mod pressure_point_geopotential;
mod runge_kutta_preparation;
#[cfg(test)]
mod test_support;

pub use column_mass_staggering::{
    ColumnMassStaggeringAxis, ColumnMassStaggeringError, ColumnMassStaggeringField,
    ColumnMassStaggeringKernels, ColumnMassStaggeringPeriodicity, ColumnMassStaggeringRegion,
    ColumnMassStaggeringResult,
};
pub use held_suarez::{
    HeldSuarezDampingAxis, HeldSuarezDampingError, HeldSuarezDampingField, HeldSuarezDampingFields,
    HeldSuarezDampingKernels, HeldSuarezDampingRegion, HeldSuarezDampingResult,
};
pub use inverse_density::{
    InverseDensityAxis, InverseDensityError, InverseDensityField, InverseDensityKernels,
    InverseDensityRegion, InverseDensityResult,
};
pub use moisture_coefficients::{
    MoistureCoefficientAxis, MoistureCoefficientError, MoistureCoefficientField,
    MoistureCoefficientKernels, MoistureCoefficientOutputs, MoistureCoefficientRegion,
    MoistureCoefficientResult, MoistureSpecies,
};
pub use momentum_coupling::{
    MomentumCouplingAxis, MomentumCouplingCoefficient, MomentumCouplingCoefficients,
    MomentumCouplingError, MomentumCouplingField, MomentumCouplingKernels,
    MomentumCouplingMapFactors, MomentumCouplingMasses, MomentumCouplingOutputs,
    MomentumCouplingRegion, MomentumCouplingResult, MomentumCouplingVelocities,
};
pub use omega_diagnosis::{
    OmegaDiagnosisAxis, OmegaDiagnosisCoefficient, OmegaDiagnosisCoefficients, OmegaDiagnosisError,
    OmegaDiagnosisField, OmegaDiagnosisGridMetrics, OmegaDiagnosisKernels,
    OmegaDiagnosisMapFactors, OmegaDiagnosisMasses, OmegaDiagnosisRegion, OmegaDiagnosisResult,
    OmegaDiagnosisVelocities,
};
pub use positive_definite::{
    PositiveDefiniteError, PositiveDefiniteKernels, PositiveDefiniteResult,
    PositiveDefiniteSlabAxis, PositiveDefiniteSlabRegion,
};
pub use pressure_point_geopotential::{
    PressurePointGeopotentialAxis, PressurePointGeopotentialError, PressurePointGeopotentialField,
    PressurePointGeopotentialKernels, PressurePointGeopotentialRegion,
    PressurePointGeopotentialResult,
};
pub use runge_kutta_preparation::{
    RungeKuttaPreparationCoefficients, RungeKuttaPreparationDiagnosticOutputs,
    RungeKuttaPreparationError, RungeKuttaPreparationInputs, RungeKuttaPreparationKernels,
    RungeKuttaPreparationMapFactors, RungeKuttaPreparationMassInputs,
    RungeKuttaPreparationMassOutputs, RungeKuttaPreparationMomentumOutputs,
    RungeKuttaPreparationOutputs, RungeKuttaPreparationRegions, RungeKuttaPreparationResult,
    RungeKuttaPreparationStage, RungeKuttaPreparationThermodynamicInputs,
    RungeKuttaPreparationVelocities,
};
