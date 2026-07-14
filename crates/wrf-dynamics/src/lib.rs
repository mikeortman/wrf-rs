//! Numerical kernels from WRF's Advanced Research WRF dynamical core.
//!
//! Each kernel family exposes a focused backend capability so CPU and future
//! GPU implementations can use native storage and execution strategies.
//! [`RungeKuttaPreparationKernels`] composes the seven translated ARW
//! diagnostics behind one failure-atomic validation boundary.
//! [`VerticalAcousticCoefficientKernels`] prepares the complete-column
//! tridiagonal factors used by the implicit vertical acoustic solve.
//! [`AcousticVerticalKernels`] consumes those factors to advance vertical
//! momentum, geopotential, and normalized time-averaged thermodynamics.
//! [`AcousticFluxAccumulationKernels`] then accumulates and finalizes the
//! staggered mass fluxes consumed by conservative scalar transport.
//! [`SpecifiedBoundaryZeroGradientKernels`] applies WRF's nearest-interior
//! copy rule to specified nonhydrostatic vertical-momentum boundaries.
//! [`SpecifiedBoundaryFlowKernels`] classifies scalar boundaries from coupled
//! U/V signs, copying outflow and clearing inflow.
//! [`SpecifiedBoundaryTendencyKernels`] assigns boundary-file tendencies before
//! relaxation and acoustic advancement.
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

mod acoustic_flux_accumulation;
mod acoustic_horizontal_momentum;
mod acoustic_mass_theta;
mod acoustic_pressure;
mod acoustic_step_preparation;
mod acoustic_trajectory;
mod acoustic_vertical_momentum;
mod column_mass_staggering;
mod dry_tendency_assembly;
mod held_suarez;
mod inverse_density;
mod moisture_coefficients;
mod momentum_coupling;
mod omega_diagnosis;
mod positive_definite;
mod pressure_point_geopotential;
mod runge_kutta_preparation;
mod specified_boundary_update;
#[cfg(test)]
mod test_support;
mod vertical_acoustic_coefficients;

pub use acoustic_flux_accumulation::{
    AcousticFluxAccumulationError, AcousticFluxAccumulationKernels, AcousticFluxAccumulationRegion,
    AcousticFluxAccumulationResult, AcousticFluxCoefficient, AcousticFluxCurrentFields,
    AcousticFluxField, AcousticFluxLinearFields, AcousticFluxMapFactors, AcousticFluxMassFields,
    AcousticFluxRunningAverages, AcousticSubstepPhase,
};
pub use acoustic_horizontal_momentum::{
    AcousticHorizontalBoundaryPolicy, AcousticHorizontalMapFactors, AcousticHorizontalMassFields,
    AcousticHorizontalMoistureCoefficients, AcousticHorizontalMomentumAxis,
    AcousticHorizontalMomentumCoefficient, AcousticHorizontalMomentumError,
    AcousticHorizontalMomentumField, AcousticHorizontalMomentumInputs,
    AcousticHorizontalMomentumKernels, AcousticHorizontalMomentumParameters,
    AcousticHorizontalMomentumRegion, AcousticHorizontalMomentumResult,
    AcousticHorizontalMomentumState, AcousticHorizontalMomentumTendencies,
    AcousticHorizontalPressureFields, AcousticHorizontalVerticalCoefficients,
    AcousticRelaxationZone, AcousticSouthNorthBoundary, AcousticWestEastBoundary,
    AcousticWestEastPeriodicity,
};
pub use acoustic_mass_theta::{
    AcousticMassThetaAxis, AcousticMassThetaBoundaryPolicy, AcousticMassThetaCoefficient,
    AcousticMassThetaDiagnostics, AcousticMassThetaError, AcousticMassThetaField,
    AcousticMassThetaInputs, AcousticMassThetaKernels, AcousticMassThetaLateralDomain,
    AcousticMassThetaMapFactors, AcousticMassThetaMassInputs, AcousticMassThetaMomentumInputs,
    AcousticMassThetaParameters, AcousticMassThetaRegion, AcousticMassThetaResult,
    AcousticMassThetaState, AcousticMassThetaThermodynamicInputs,
    AcousticMassThetaVerticalCoefficients, AcousticMassThetaWestEastPeriodicity,
};
pub use acoustic_pressure::{
    AcousticPressureAxis, AcousticPressureCoefficient, AcousticPressureCoefficients,
    AcousticPressureDampingPhase, AcousticPressureError, AcousticPressureField,
    AcousticPressureKernels, AcousticPressureMasses, AcousticPressureMode,
    AcousticPressureParameters, AcousticPressureRegion, AcousticPressureResult,
    AcousticPressureState, AcousticPressureThermodynamics, AcousticPressureVerticalMetrics,
};
pub use acoustic_step_preparation::{
    AcousticStepPreparationAxis, AcousticStepPreparationCoefficient,
    AcousticStepPreparationCoefficients, AcousticStepPreparationColumnMassTimeLevels,
    AcousticStepPreparationDiagnosticInputs, AcousticStepPreparationError,
    AcousticStepPreparationField, AcousticStepPreparationKernels,
    AcousticStepPreparationMapFactors, AcousticStepPreparationMassInputs,
    AcousticStepPreparationMassOutputs, AcousticStepPreparationPhase,
    AcousticStepPreparationRegion, AcousticStepPreparationResult,
    AcousticStepPreparationSavedOutputs, AcousticStepPreparationVolumeTimeLevels,
};
pub use acoustic_trajectory::{
    AcousticTrajectoryCoefficients, AcousticTrajectoryControls, AcousticTrajectoryDiagnostics,
    AcousticTrajectoryError, AcousticTrajectoryInputs, AcousticTrajectoryKernels,
    AcousticTrajectoryMapFactors, AcousticTrajectoryMassInputs,
    AcousticTrajectoryMoistureCoefficients, AcousticTrajectoryPressureInputs,
    AcousticTrajectoryRegions, AcousticTrajectoryResult, AcousticTrajectorySavedState,
    AcousticTrajectoryTendencies, AcousticTrajectoryTimeLevels, AcousticTrajectoryWorkspace,
};
pub use acoustic_vertical_momentum::{
    AcousticVerticalAdvection, AcousticVerticalAxis, AcousticVerticalBoundaryPolicy,
    AcousticVerticalCoefficient, AcousticVerticalDamping, AcousticVerticalError,
    AcousticVerticalField, AcousticVerticalGeopotentialInputs, AcousticVerticalInputs,
    AcousticVerticalKernels, AcousticVerticalLateralDomain, AcousticVerticalLevelCoefficients,
    AcousticVerticalMapFactors, AcousticVerticalMassInputs, AcousticVerticalMomentumInputs,
    AcousticVerticalParameters, AcousticVerticalRegion, AcousticVerticalResult,
    AcousticVerticalSolveInputs, AcousticVerticalState, AcousticVerticalThermodynamicInputs,
    AcousticVerticalWestEastPeriodicity, AcousticVerticalWorkspace,
};
pub use column_mass_staggering::{
    ColumnMassStaggeringAxis, ColumnMassStaggeringError, ColumnMassStaggeringField,
    ColumnMassStaggeringKernels, ColumnMassStaggeringPeriodicity, ColumnMassStaggeringRegion,
    ColumnMassStaggeringResult,
};
pub use dry_tendency_assembly::{
    DryTendencyAssemblyAxis, DryTendencyAssemblyCoefficient, DryTendencyAssemblyCoefficients,
    DryTendencyAssemblyError, DryTendencyAssemblyField, DryTendencyAssemblyForwardTendencies,
    DryTendencyAssemblyKernels, DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase,
    DryTendencyAssemblyRegion, DryTendencyAssemblyResult, DryTendencyAssemblyRungeKuttaTendencies,
    DryTendencyAssemblySavedTendencies, DryTendencyAssemblyThermodynamics,
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
pub use specified_boundary_update::{
    SpecifiedBoundaryFieldLocation, SpecifiedBoundaryFinalizationBoundaryFields,
    SpecifiedBoundaryFinalizationError, SpecifiedBoundaryFinalizationFieldLocation,
    SpecifiedBoundaryFinalizationInputs, SpecifiedBoundaryFinalizationKernels,
    SpecifiedBoundaryFinalizationParameters, SpecifiedBoundaryFinalizationRegion,
    SpecifiedBoundaryFinalizationResult, SpecifiedBoundaryFlowError, SpecifiedBoundaryFlowField,
    SpecifiedBoundaryFlowInputs, SpecifiedBoundaryFlowKernels, SpecifiedBoundaryFlowParameters,
    SpecifiedBoundaryFlowRegion, SpecifiedBoundaryFlowResult, SpecifiedBoundaryGeopotentialError,
    SpecifiedBoundaryGeopotentialInputs, SpecifiedBoundaryGeopotentialKernels,
    SpecifiedBoundaryGeopotentialResult, SpecifiedBoundaryInflowPolicy,
    SpecifiedBoundaryTendencies, SpecifiedBoundaryTendencyError, SpecifiedBoundaryTendencyKernels,
    SpecifiedBoundaryTendencyParameters, SpecifiedBoundaryTendencyResult,
    SpecifiedBoundaryUpdateAxis, SpecifiedBoundaryUpdateError, SpecifiedBoundaryUpdateKernels,
    SpecifiedBoundaryUpdateParameters, SpecifiedBoundaryUpdateRegion,
    SpecifiedBoundaryUpdateResult, SpecifiedBoundaryWestEastPeriodicity,
    SpecifiedBoundaryZeroGradientError, SpecifiedBoundaryZeroGradientKernels,
    SpecifiedBoundaryZeroGradientParameters, SpecifiedBoundaryZeroGradientResult,
};
pub use vertical_acoustic_coefficients::{
    VerticalAcousticCoefficient, VerticalAcousticCoefficientAxis, VerticalAcousticCoefficientError,
    VerticalAcousticCoefficientField, VerticalAcousticCoefficientInputs,
    VerticalAcousticCoefficientKernels, VerticalAcousticCoefficientParameters,
    VerticalAcousticCoefficientRegion, VerticalAcousticCoefficientResult,
    VerticalAcousticMassCoefficients, VerticalAcousticMetrics, VerticalAcousticSolveCoefficients,
    VerticalAcousticTopBoundary,
};
