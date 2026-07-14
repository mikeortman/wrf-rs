use std::error::Error;
use std::fmt;

use wrf_compute::GridShape;

use crate::{
    ColumnMassStaggeringError, InverseDensityError, MoistureCoefficientError,
    MomentumCouplingError, OmegaDiagnosisError, PressurePointGeopotentialError,
    RungeKuttaPreparationStage,
};

/// Result returned by integrated Runge-Kutta preparation.
pub type RungeKuttaPreparationResult<Value> = Result<Value, RungeKuttaPreparationError>;

/// Typed validation or execution failure from one preparation stage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RungeKuttaPreparationError {
    /// One component region describes a different grid from momentum coupling.
    RegionShapeMismatch {
        /// Component whose region shape is inconsistent.
        stage: RungeKuttaPreparationStage,
        /// Shape established by the momentum-coupling region.
        expected: GridShape,
        /// Shape supplied for this component.
        actual: GridShape,
    },
    /// Full or staggered dry-air column-mass preparation failed.
    ColumnMass(ColumnMassStaggeringError),
    /// Mass coupling of the three momentum components failed.
    MomentumCoupling(MomentumCouplingError),
    /// Dry-air omega diagnosis failed.
    OmegaDiagnosis(OmegaDiagnosisError),
    /// Momentum-staggered moisture coefficients failed.
    MoistureCoefficients(MoistureCoefficientError),
    /// Full inverse-density calculation failed.
    InverseDensity(InverseDensityError),
    /// Pressure-point geopotential calculation failed.
    PressurePointGeopotential(PressurePointGeopotentialError),
}

impl fmt::Display for RungeKuttaPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegionShapeMismatch {
                stage,
                expected,
                actual,
            } => write!(
                formatter,
                "{stage} region shape {actual:?} differs from integrated shape {expected:?}"
            ),
            Self::ColumnMass(error) => write!(formatter, "column-mass preparation failed: {error}"),
            Self::MomentumCoupling(error) => {
                write!(formatter, "momentum coupling failed: {error}")
            }
            Self::OmegaDiagnosis(error) => write!(formatter, "omega diagnosis failed: {error}"),
            Self::MoistureCoefficients(error) => {
                write!(formatter, "moisture coefficients failed: {error}")
            }
            Self::InverseDensity(error) => write!(formatter, "inverse density failed: {error}"),
            Self::PressurePointGeopotential(error) => {
                write!(formatter, "pressure-point geopotential failed: {error}")
            }
        }
    }
}

impl Error for RungeKuttaPreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RegionShapeMismatch { .. } => None,
            Self::ColumnMass(error) => Some(error),
            Self::MomentumCoupling(error) => Some(error),
            Self::OmegaDiagnosis(error) => Some(error),
            Self::MoistureCoefficients(error) => Some(error),
            Self::InverseDensity(error) => Some(error),
            Self::PressurePointGeopotential(error) => Some(error),
        }
    }
}

impl From<ColumnMassStaggeringError> for RungeKuttaPreparationError {
    fn from(error: ColumnMassStaggeringError) -> Self {
        Self::ColumnMass(error)
    }
}

impl From<MomentumCouplingError> for RungeKuttaPreparationError {
    fn from(error: MomentumCouplingError) -> Self {
        Self::MomentumCoupling(error)
    }
}

impl From<OmegaDiagnosisError> for RungeKuttaPreparationError {
    fn from(error: OmegaDiagnosisError) -> Self {
        Self::OmegaDiagnosis(error)
    }
}

impl From<MoistureCoefficientError> for RungeKuttaPreparationError {
    fn from(error: MoistureCoefficientError) -> Self {
        Self::MoistureCoefficients(error)
    }
}

impl From<InverseDensityError> for RungeKuttaPreparationError {
    fn from(error: InverseDensityError) -> Self {
        Self::InverseDensity(error)
    }
}

impl From<PressurePointGeopotentialError> for RungeKuttaPreparationError {
    fn from(error: PressurePointGeopotentialError) -> Self {
        Self::PressurePointGeopotential(error)
    }
}
